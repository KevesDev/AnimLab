import React, { useEffect, useRef, useState } from 'react';
import init_core, { AnimLabEngine } from 'animlab-core';
import { usePreferencesStore } from '../store/PreferencesStore';
import { GlobalInputManager } from '../engine_bridge/InputManager';

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
                
                const dpr = window.devicePixelRatio || 1;
                const rect = containerRef.current.getBoundingClientRect();
                const physicalWidth = Math.max(1, Math.floor(rect.width * dpr));
                const physicalHeight = Math.max(1, Math.floor(rect.height * dpr));

                canvasRef.current.width = physicalWidth;
                canvasRef.current.height = physicalHeight;

                await engine.attach_canvas(canvasRef.current, physicalWidth, physicalHeight);
                
                setEngineInstance(engine);
                
                // AAA ARCHITECTURE: Hand the canvas over to the Global Input Manager natively.
                GlobalInputManager.getInstance().attachCanvas(canvasRef.current);
                
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

        const resizeObserver = new ResizeObserver((entries) => {
            if (!engineRef.current || !canvasRef.current) return;
            
            for (let entry of entries) {
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
                }, 100); 
            }
        });

        if (containerRef.current) resizeObserver.observe(containerRef.current);

        return () => {
            isMounted = false;
            cancelAnimationFrame(animationFrameId.current);
            if (resizeTimeoutRef.current) window.clearTimeout(resizeTimeoutRef.current);
            resizeObserver.disconnect();
            
            // Clean up the native DOM event hooks
            GlobalInputManager.getInstance().detachCanvas();
            
            if (engineRef.current) {
                engineRef.current.free();
                setEngineInstance(null);
            }
        };
    }, [setEngineInstance]);

    return (
        <div ref={containerRef} style={{ width: '100%', height: '100%', position: 'relative', overflow: 'hidden', backgroundColor: '#141517' }}>
            {isBooting && (
                <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)', color: '#888', fontFamily: 'sans-serif' }}>
                    Initializing WebGPU Pipeline...
                </div>
            )}
            {/* The React Component is now completely "dumb". No pointer events are bound here. */}
            <canvas
                ref={canvasRef}
                style={{ display: 'block', width: '100%', height: '100%', touchAction: 'none' }}
            />
        </div>
    );
};