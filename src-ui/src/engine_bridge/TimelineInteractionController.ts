import { usePreferencesStore } from '../store/PreferencesStore';

export enum TimelineMode { Idle, Scrubbing, Extending }

export class TimelineInteractionController {
    private static instance: TimelineInteractionController;
    public mode: TimelineMode = TimelineMode.Idle;
    private frameWidth = 16;
    
    private activeElementId: bigint | null = null;
    private activeStartFrame: number = 0;

    private constructor() {}

    public static getInstance(): TimelineInteractionController {
        if (!TimelineInteractionController.instance) {
            TimelineInteractionController.instance = new TimelineInteractionController();
        }
        return TimelineInteractionController.instance;
    }

    // Helper: Gets absolute positioning directly from the DOM to bypass React State lag
    private getMetrics(target: HTMLElement) {
        const container = target.closest('.timeline-scroll-container') as HTMLElement;
        if (!container) return { left: 0, scroll: 0 };
        return { left: container.getBoundingClientRect().left, scroll: container.scrollLeft };
    }

    // --- Playhead Scrubbing ---
    public handleRulerPointerDown = (e: React.PointerEvent) => {
        e.preventDefault();
        this.mode = TimelineMode.Scrubbing;
        const target = e.currentTarget as HTMLElement;
        target.setPointerCapture(e.pointerId);
        
        const { left, scroll } = this.getMetrics(target);
        this.updateScrub(e.clientX, left, scroll);
    };

    public handleRulerPointerMove = (e: React.PointerEvent) => {
        if (this.mode === TimelineMode.Scrubbing) {
            const target = e.currentTarget as HTMLElement;
            const { left, scroll } = this.getMetrics(target);
            this.updateScrub(e.clientX, left, scroll);
        }
    };

    public handleRulerPointerUp = (e: React.PointerEvent) => {
        this.mode = TimelineMode.Idle;
        const target = e.currentTarget as HTMLElement;
        if (target.hasPointerCapture(e.pointerId)) target.releasePointerCapture(e.pointerId);
    };

    private updateScrub(clientX: number, containerLeft: number, scrollLeft: number) {
        const x = clientX - containerLeft + scrollLeft;
        const frame = Math.max(1, Math.floor(x / this.frameWidth) + 1);
        usePreferencesStore.getState().setCurrentFrame(frame);
    }

    // --- Harmony Block Extending ---
    public handleBlockEdgePointerDown = (e: React.PointerEvent, elementId: bigint, startFrame: number) => {
        e.preventDefault();
        e.stopPropagation();
        this.mode = TimelineMode.Extending;
        this.activeElementId = elementId;
        this.activeStartFrame = startFrame;
        
        // Secure global capture so dragging outside the div doesn't drop the event
        const target = e.currentTarget as HTMLElement;
        target.setPointerCapture(e.pointerId);
    };

    public handleBlockEdgePointerMove = (e: React.PointerEvent) => {
        if (this.mode === TimelineMode.Extending && this.activeElementId !== null) {
            const target = e.currentTarget as HTMLElement;
            const { left, scroll } = this.getMetrics(target);
            
            const x = e.clientX - left + scroll;
            const currentHoverFrame = Math.max(1, Math.floor(x / this.frameWidth) + 1);
            const newDuration = Math.max(1, currentHoverFrame - this.activeStartFrame + 1);
            
            const prefs = usePreferencesStore.getState();
            const engine = prefs.engineInstance;
            if (engine) {
                // Update Rust Engine instantly
                engine.extend_exposure(this.activeElementId, this.activeStartFrame, newDuration);
                // Sync UI
                prefs.fetchTimelineState();
            }
        }
    };

    public handleBlockEdgePointerUp = (e: React.PointerEvent) => {
        this.mode = TimelineMode.Idle;
        this.activeElementId = null;
        const target = e.currentTarget as HTMLElement;
        if (target.hasPointerCapture(e.pointerId)) target.releasePointerCapture(e.pointerId);
    };
}