import React, { useEffect, useRef, useState } from 'react';
import init_core, { AnimLabEngine } from 'animlab-core';
import { usePreferencesStore } from '../store/preferencesStore';

export const CanvasViewport: React.FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [isBooting, setIsBooting] = useState(true);
    
    // Access the global store to register the engine once initialized
    const setEngineInstance = usePreferencesStore(state => state.setEngineInstance);
    const engineRef = useRef<any>(null);
    const animationFrameId = useRef<number>(0);

    useEffect(() => {
        let isMounted = true;

        const bootEngine = async () => {
            try {
                // Initialize the WASM binary memory
                await init_core();
                
                if (!isMounted || !canvasRef.current || !containerRef.current) return;

                const engine = new AnimLabEngine();
                engineRef.current = engine;
                
                // Read physical dimensions of the FlexLayout container
                const rect = containerRef.current.getBoundingClientRect();
                canvasRef.current.width = rect.width;
                canvasRef.current.height = rect.height;

                // Handshake with WebGPU
                await engine.attach_canvas(canvasRef.current, rect.width, rect.height);
                
                // Register the engine globally so the UI can command it
                setEngineInstance(engine);
                setIsBooting(false);

                // Ignite the 60fps Native Render Loop
                const renderLoop = () => {
                    if (engineRef.current) {
                        engineRef.current.render();
                    }
                    animationFrameId.current = requestAnimationFrame(renderLoop);
                };
                renderLoop();

            } catch (err) {
                console.error("AnimLab Fatal Graphics Error:", err);
            }
        };

        bootEngine();

        return () => {
            isMounted = false;
            cancelAnimationFrame(animationFrameId.current);
            if (engineRef.current) {
                engineRef.current.free();
                setEngineInstance(null);
            }
        };
    }, [setEngineInstance]);

    // --- HARDWARE INPUT ROUTING ---
    const handlePointerDown = (e: React.PointerEvent<HTMLCanvasElement>) => {
        if (!engineRef.current || !canvasRef.current) return;
        canvasRef.current.setPointerCapture(e.pointerId);
        
        // Calculate coordinate space relative to the moving FlexLayout panel
        const rect = canvasRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const pressure = e.pressure !== 0 ? e.pressure : 1.0;
        
        engineRef.current.begin_stroke(x, y, pressure);
    };

    const handlePointerMove = (e: React.PointerEvent<HTMLCanvasElement>) => {
        if (!engineRef.current || !canvasRef.current || !canvasRef.current.hasPointerCapture(e.pointerId)) return;
        
        const rect = canvasRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const pressure = e.pressure !== 0 ? e.pressure : 1.0;
        
        engineRef.current.push_point(x, y, pressure);
    };

    const handlePointerUp = (e: React.PointerEvent<HTMLCanvasElement>) => {
        if (!engineRef.current || !canvasRef.current || !canvasRef.current.hasPointerCapture(e.pointerId)) return;
        canvasRef.current.releasePointerCapture(e.pointerId);
        
        engineRef.current.end_stroke();
    };

    return (
        <div ref={containerRef} style={{ width: '100%', height: '100%', position: 'relative', overflow: 'hidden', backgroundColor: '#141517' }}>
            {isBooting && (
                <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)', color: '#888', fontFamily: 'sans-serif' }}>
                    Initializing WebGPU Pipeline...
                </div>
            )}
            <canvas
                ref={canvasRef}
                style={{ display: 'block', width: '100%', height: '100%', touchAction: 'none' }}
                onPointerDown={handlePointerDown}
                onPointerMove={handlePointerMove}
                onPointerUp={handlePointerUp}
                onPointerCancel={handlePointerUp}
            />
        </div>
    );
};