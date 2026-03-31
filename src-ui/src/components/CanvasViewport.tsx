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

        const resizeObserver = new ResizeObserver((entries) => {
            for (let entry of entries) {
                const { width, height } = entry.contentRect;
                
                // AAA FIX: Do not attempt to mount WebGPU if the FlexLayout container is zero-sized
                if (width <= 0 || height <= 0) return;

                const dpr = window.devicePixelRatio || 1;
                canvas.width = width * dpr;
                canvas.height = height * dpr;

                if (!attached) {
                    engineInstance.attach_canvas(canvas, canvas.width, canvas.height).then(() => {
                        engineInstance.render();
                        inputManager.attachCanvas(canvas);
                        attached = true;
                    });
                } else {
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
        <div ref={containerRef} style={{ position: 'absolute', inset: 0, backgroundColor: '#0a0a0c', overflow: 'hidden' }}>
            <canvas 
                ref={canvasRef} 
                style={{ width: '100%', height: '100%', cursor: 'crosshair', touchAction: 'none' }}
                onContextMenu={(e) => e.preventDefault()}
            />
        </div>
    );
};