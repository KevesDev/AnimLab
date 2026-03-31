import React, { useEffect, useRef } from 'react';
import { GlobalInputManager } from '../engine_bridge/InputManager';
import { usePreferencesStore } from '../store/PreferencesStore';

export const CanvasViewport: React.FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const { engineInstance } = usePreferencesStore();

    useEffect(() => {
        if (!canvasRef.current || !containerRef.current || !engineInstance) return;
        const canvas = canvasRef.current;
        const container = containerRef.current;
        const inputManager = GlobalInputManager.getInstance();
        
        let attached = false;
        let isAttaching = false;

        const resizeObserver = new ResizeObserver((entries) => {
            for (let entry of entries) {
                const { width, height } = entry.contentRect;
                if (width <= 5 || height <= 5) return; 
                
                const dpr = window.devicePixelRatio || 1;
                canvas.width = width * dpr;
                canvas.height = height * dpr;

                if (!attached && !isAttaching) {
                    isAttaching = true;
                    
                    engineInstance.attach_canvas(canvas, canvas.width, canvas.height)
                        .then(() => {
                            engineInstance.render();
                            inputManager.attachCanvas(canvas);
                            attached = true;
                            isAttaching = false;
                        })
                        .catch((err: any) => {
                            console.error("AnimLab Engine failed to attach:", err);
                            isAttaching = false;
                        });
                } else if (attached) {
                    engineInstance.resize_surface(canvas.width, canvas.height);
                    engineInstance.render();
                }
            }
        });
        
        resizeObserver.observe(container);
        
        return () => { 
            resizeObserver.disconnect(); 
            inputManager.detachCanvas(); 
        };
    }, [engineInstance]);

    return (
        <div ref={containerRef} style={{ position: 'absolute', inset: 0, backgroundColor: '#141517', overflow: 'hidden' }}>
            <canvas 
                id="animlab-canvas"
                ref={canvasRef} 
                // AAA FIX: Added a light 'paper' background color so you can actually see your strokes
                style={{ width: '100%', height: '100%', cursor: 'crosshair', touchAction: 'none', backgroundColor: '#e5e5e5' }} 
            />
        </div>
    );
};