import React, { useEffect, useRef, useState } from 'react';
import init_core, { AnimLabEngine } from 'animlab-core';
import { usePreferencesStore } from '../store/preferencesStore';

export const CanvasViewport: React.FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const [isBooting, setIsBooting] = useState(true);
    
    const setEngineInstance = usePreferencesStore(state => state.setEngineInstance);
    const engineRef = useRef<any>(null);
    const animationFrameId = useRef<number>(0);
    const resizeTimeoutRef = useRef<number | null>(null);

    useEffect(() => {
        let isMounted = true;

        const bootEngine = async () => {
            try {
                await init_core();
                if (!isMounted || !canvasRef.current || !containerRef.current) return;

                const engine = new AnimLabEngine();
                engineRef.current = engine;
                
                // Calculate actual hardware pixels (supports 4K and Retina Displays)
                const dpr = window.devicePixelRatio || 1;
                const rect = containerRef.current.getBoundingClientRect();
                const physicalWidth = Math.max(1, Math.floor(rect.width * dpr));
                const physicalHeight = Math.max(1, Math.floor(rect.height * dpr));

                // Force the HTML canvas to hold the physical pixel count
                canvasRef.current.width = physicalWidth;
                canvasRef.current.height = physicalHeight;

                await engine.attach_canvas(canvasRef.current, physicalWidth, physicalHeight);
                
                setEngineInstance(engine);
                setIsBooting(false);

                const renderLoop = () => {
                    if (engineRef.current) { engineRef.current.render(); }
                    animationFrameId.current = requestAnimationFrame(renderLoop);
                };
                renderLoop();

            } catch (err) {
                console.error("AnimLab Fatal Graphics Error:", err);
            }
        };

        bootEngine();

        // AAA FIX: Hardware Swapchain Resize Observer
        const resizeObserver = new ResizeObserver((entries) => {
            if (!engineRef.current || !canvasRef.current) return;
            
            for (let entry of entries) {
                // Debounce the resize to prevent GPU Swapchain Exhaustion during UI dragging
                if (resizeTimeoutRef.current) window.clearTimeout(resizeTimeoutRef.current);
                
                resizeTimeoutRef.current = window.setTimeout(() => {
                    const rect = entry.target.getBoundingClientRect();
                    const dpr = window.devicePixelRatio || 1;
                    const pWidth = Math.max(1, Math.floor(rect.width * dpr));
                    const pHeight = Math.max(1, Math.floor(rect.height * dpr));

                    if (canvasRef.current && (canvasRef.current.width !== pWidth || canvasRef.current.height !== pHeight)) {
                        canvasRef.current.width = pWidth;
                        canvasRef.current.height = pHeight;
                        engineRef.current.resize_surface(pWidth, pHeight);
                    }
                }, 100); // Wait 100ms for layout to settle before reallocating VRAM
            }
        });

        if (containerRef.current) resizeObserver.observe(containerRef.current);

        return () => {
            isMounted = false;
            cancelAnimationFrame(animationFrameId.current);
            if (resizeTimeoutRef.current) window.clearTimeout(resizeTimeoutRef.current);
            resizeObserver.disconnect();
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
        
        const rect = canvasRef.current.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        
        // AAA FIX: Multiply CSS coordinates by the DPI to hit the exact hardware pixel
        const x = (e.clientX - rect.left) * dpr;
        const y = (e.clientY - rect.top) * dpr;
        const pressure = e.pressure !== 0 ? e.pressure : 1.0;
        
        engineRef.current.begin_stroke(x, y, pressure);
    };

    const handlePointerMove = (e: React.PointerEvent<HTMLCanvasElement>) => {
        if (!engineRef.current || !canvasRef.current || !canvasRef.current.hasPointerCapture(e.pointerId)) return;
        
        const rect = canvasRef.current.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        
        const x = (e.clientX - rect.left) * dpr;
        const y = (e.clientY - rect.top) * dpr;
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