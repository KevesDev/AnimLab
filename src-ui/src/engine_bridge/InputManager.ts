import { AnimLabEngine } from 'animlab-core';

/// <summary>
/// A pure TypeScript class that acts as the Single Source of Truth for all canvas interactions.
/// It intercepts PointerEvents, manages pointer capture locks, and pipes data directly to the Rust engine.
/// </summary>
export class GlobalInputManager {
    private canvas: HTMLCanvasElement;
    private engine: AnimLabEngine;
    private isDrawing: boolean = false;

    constructor(canvas: HTMLCanvasElement, engine: AnimLabEngine) {
        this.canvas = canvas;
        this.engine = engine;

        // Bind the event listeners to the class instance
        this.handlePointerDown = this.handlePointerDown.bind(this);
        this.handlePointerMove = this.handlePointerMove.bind(this);
        this.handlePointerUp = this.handlePointerUp.bind(this);

        // Attach listeners directly to the physical WebGPU canvas
        // We use 'pointer' events instead of 'mouse' events because pointer natively handles 
        // Wacom tablets, Apple Pencils, and standard mice all through one unified API.
        this.canvas.addEventListener('pointerdown', this.handlePointerDown);
        this.canvas.addEventListener('pointermove', this.handlePointerMove);
        this.canvas.addEventListener('pointerup', this.handlePointerUp);
        this.canvas.addEventListener('pointercancel', this.handlePointerUp); // Handles window losing focus securely
        this.canvas.addEventListener('pointerout', this.handlePointerUp);
    }

    private handlePointerDown(e: PointerEvent): void {
        // Only accept primary button inputs (Left Click or Pen Tip)
        if (e.button !== 0) return;

        try {
            this.isDrawing = true;
            
            // Lock the pointer to the canvas. This guarantees we keep receiving coordinates
            // even if the user drags their pen wildly outside the application window.
            this.canvas.setPointerCapture(e.pointerId);

            // Coerce the pressure value. Standard mice do not report pressure, so we default to 1.0 (max).
            const pressure = e.pressure !== 0 ? e.pressure : 1.0;
            
            this.engine.begin_stroke(e.offsetX, e.offsetY, pressure);
        } catch (error) {
            console.error("[InputManager] Fatal error initiating stroke pipeline:", error);
            this.isDrawing = false; // Failsafe state reset
        }
    }

    private handlePointerMove(e: PointerEvent): void {
        if (!this.isDrawing) return;

        try {
            const pressure = e.pressure !== 0 ? e.pressure : 1.0;
            // Coalesced events (multiple hardware ticks per browser frame) can be added here later for extreme precision
            this.engine.push_point(e.offsetX, e.offsetY, pressure);
        } catch (error) {
            console.error("[InputManager] Fatal error piping point data to Engine:", error);
            this.isDrawing = false;
        }
    }

    private handlePointerUp(e: PointerEvent): void {
        if (!this.isDrawing) return;

        try {
            this.isDrawing = false;
            
            // Release the hardware lock securely
            this.canvas.releasePointerCapture(e.pointerId);
            
            this.engine.end_stroke();
        } catch (error) {
            console.error("[InputManager] Fatal error terminating stroke pipeline:", error);
        }
    }

    /// <summary>
    /// ZERO-DEBT RULE: Always sever DOM listeners when the module is destroyed to prevent memory leaks.
    /// </summary>
    public dispose(): void {
        this.canvas.removeEventListener('pointerdown', this.handlePointerDown);
        this.canvas.removeEventListener('pointermove', this.handlePointerMove);
        this.canvas.removeEventListener('pointerup', this.handlePointerUp);
        this.canvas.removeEventListener('pointercancel', this.handlePointerUp);
        this.canvas.removeEventListener('pointerout', this.handlePointerUp);
    }
}