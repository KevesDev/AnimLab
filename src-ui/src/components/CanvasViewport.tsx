import React, { useRef, useEffect } from 'react';
import { usePreferencesStore } from '../store/PreferencesStore';

// AAA FIX: Changed to a named export to match App.tsx
export const CanvasViewport: React.FC = () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);
    const engineRef = useRef<any>(null);

    const activeTool = usePreferencesStore((state) => state.activeTool);
    const brushColor = usePreferencesStore((state) => state.brushColor);
    const brushSize = usePreferencesStore((state) => state.brushSize);

    useEffect(() => {
        let isMounted = true;

        const initEngine = async () => {
            if (!canvasRef.current || !containerRef.current) return;
            
            try {
                const wasm = await import('animlab-core');
                await wasm.default();
                
                if (isMounted && !engineRef.current) {
                    const engine = new wasm.AnimLabEngine();
                    
                    const rect = containerRef.current.getBoundingClientRect();
                    canvasRef.current.width = rect.width;
                    canvasRef.current.height = rect.height;
                    
                    await engine.attach_canvas(canvasRef.current, rect.width, rect.height);
                    engine.set_active_tool(activeTool);
                    engine.set_brush_settings(brushSize, brushColor.r / 255, brushColor.g / 255, brushColor.b / 255, brushColor.a);
                    
                    engineRef.current = engine;
                    
                    const renderLoop = () => {
                        if (isMounted && engineRef.current) {
                            engineRef.current.render();
                            requestAnimationFrame(renderLoop);
                        }
                    };
                    requestAnimationFrame(renderLoop);
                }
            } catch (err) {
                console.error("Failed to initialize AnimLab Engine:", err);
            }
        };

        initEngine();

        const handleResize = () => {
            if (containerRef.current && canvasRef.current && engineRef.current) {
                const rect = containerRef.current.getBoundingClientRect();
                canvasRef.current.width = rect.width;
                canvasRef.current.height = rect.height;
                engineRef.current.resize_surface(rect.width, rect.height);
            }
        };

        window.addEventListener('resize', handleResize);

        return () => {
            isMounted = false;
            window.removeEventListener('resize', handleResize);
            if (engineRef.current) {
                engineRef.current.free();
                engineRef.current = null;
            }
        };
    }, []);

    useEffect(() => {
        if (engineRef.current) { engineRef.current.set_active_tool(activeTool); }
    }, [activeTool]);

    useEffect(() => {
        if (engineRef.current) {
            engineRef.current.set_brush_settings(brushSize, brushColor.r / 255, brushColor.g / 255, brushColor.b / 255, brushColor.a);
        }
    }, [brushColor, brushSize]);

    const handlePointerDown = (e: React.PointerEvent<HTMLCanvasElement>) => {
        if (!engineRef.current || !canvasRef.current) return;
        const rect = canvasRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        const pressure = e.pointerType === 'pen' ? e.pressure : 1.0;
        canvasRef.current.setPointerCapture(e.pointerId);
        engineRef.current.begin_stroke(x, y, pressure);
    };

    const handlePointerMove = (e: React.PointerEvent<HTMLCanvasElement>) => {
        if (!engineRef.current || !canvasRef.current) return;
        const rect = canvasRef.current.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        
        if (e.buttons === 0) {
            engineRef.current.hover(x, y);
        } else {
            const pressure = e.pointerType === 'pen' ? e.pressure : 1.0;
            engineRef.current.push_point(x, y, pressure);
        }
    };

    const handlePointerUp = (e: React.PointerEvent<HTMLCanvasElement>) => {
        if (!engineRef.current || !canvasRef.current) return;
        canvasRef.current.releasePointerCapture(e.pointerId);
        engineRef.current.end_stroke();
    };

    return (
        <div ref={containerRef} className="flex-1 w-full h-full relative overflow-hidden bg-[#14161a]">
            <canvas
                ref={canvasRef}
                className="absolute top-0 left-0 w-full h-full touch-none"
                onPointerDown={handlePointerDown}
                onPointerMove={handlePointerMove}
                onPointerUp={handlePointerUp}
                onPointerCancel={handlePointerUp}
                onPointerLeave={handlePointerUp}
                onContextMenu={(e) => e.preventDefault()}
            />
        </div>
    );
};