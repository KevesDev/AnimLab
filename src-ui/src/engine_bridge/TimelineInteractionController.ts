import { usePreferencesStore } from '../store/PreferencesStore';

export enum TimelineMode { Idle, Scrubbing, ExtendingRight, ExtendingLeft, Moving }

export class TimelineInteractionController {
    private static instance: TimelineInteractionController;
    public mode: TimelineMode = TimelineMode.Idle;
    public frameWidth = 16;
    
    public activeElementId: bigint | null = null;
    public activeStartFrame: number = 0;
    public activeDuration: number = 0;
    
    private initialMouseX: number = 0;

    private constructor() {
        // Bind global methods to preserve 'this' context
        this.handleGlobalRulerMove = this.handleGlobalRulerMove.bind(this);
        this.handleGlobalRulerUp = this.handleGlobalRulerUp.bind(this);
        this.handleGlobalBlockMove = this.handleGlobalBlockMove.bind(this);
        this.handleGlobalBlockUp = this.handleGlobalBlockUp.bind(this);
    }

    public static getInstance(): TimelineInteractionController {
        if (!TimelineInteractionController.instance) {
            TimelineInteractionController.instance = new TimelineInteractionController();
        }
        return TimelineInteractionController.instance;
    }

    // AAA FIX: Query the container globally to avoid stale React event targets during a drag
    private getContainerMetrics() {
        const container = document.querySelector('.timeline-scroll-container') as HTMLElement;
        if (!container) return { left: 0, scroll: 0 };
        return { left: container.getBoundingClientRect().left, scroll: container.scrollLeft };
    }

    // --- Scrubbing ---
    public handleRulerPointerDown = (e: React.PointerEvent) => {
        e.preventDefault();
        this.mode = TimelineMode.Scrubbing;
        
        const { left, scroll } = this.getContainerMetrics();
        this.updateScrub(e.clientX, left, scroll);

        // AAA FIX: Attach to the Window so dragging off the ruler doesn't break
        window.addEventListener('pointermove', this.handleGlobalRulerMove);
        window.addEventListener('pointerup', this.handleGlobalRulerUp);
    };

    private handleGlobalRulerMove(e: PointerEvent) {
        if (this.mode === TimelineMode.Scrubbing) {
            const { left, scroll } = this.getContainerMetrics();
            this.updateScrub(e.clientX, left, scroll);
        }
    }

    private handleGlobalRulerUp(e: PointerEvent) {
        this.mode = TimelineMode.Idle;
        window.removeEventListener('pointermove', this.handleGlobalRulerMove);
        window.removeEventListener('pointerup', this.handleGlobalRulerUp);
    }

    private updateScrub(clientX: number, containerLeft: number, scrollLeft: number) {
        const x = clientX - containerLeft + scrollLeft;
        const frame = Math.max(1, Math.floor(x / this.frameWidth) + 1);
        usePreferencesStore.getState().setCurrentFrame(frame);
    }

    // --- Block Manipulation Initiation ---
    public handleBlockPointerDown = (mode: TimelineMode, e: React.PointerEvent, elementId: bigint, startFrame: number, duration: number) => {
        e.preventDefault(); e.stopPropagation();
        
        const store = usePreferencesStore.getState();
        store.setCurrentFrame(startFrame);
        store.setSelectedLayerId(elementId);

        this.mode = mode;
        this.activeElementId = elementId;
        this.activeStartFrame = startFrame;
        this.activeDuration = duration;
        
        const { left, scroll } = this.getContainerMetrics();
        this.initialMouseX = e.clientX - left + scroll;
        
        // AAA FIX: Global listeners ensure the Ghost Drag never drops
        window.addEventListener('pointermove', this.handleGlobalBlockMove);
        window.addEventListener('pointerup', this.handleGlobalBlockUp);
        
        store.setGhostState({
            elementId, originalStart: startFrame, originalDuration: duration, newStart: startFrame, newDuration: duration
        });
    };

    // --- The Math Engine (Global) ---
    private handleGlobalBlockMove(e: PointerEvent) {
        if (this.mode !== TimelineMode.Idle && this.mode !== TimelineMode.Scrubbing && this.activeElementId !== null) {
            const { left, scroll } = this.getContainerMetrics();
            const currentMouseX = e.clientX - left + scroll;
            
            const deltaPixels = currentMouseX - this.initialMouseX;
            const deltaFrames = Math.round(deltaPixels / this.frameWidth);
            
            let newStart = this.activeStartFrame;
            let newDuration = this.activeDuration;

            if (this.mode === TimelineMode.Moving) {
                newStart = Math.max(1, this.activeStartFrame + deltaFrames);
            } 
            else if (this.mode === TimelineMode.ExtendingRight) {
                newDuration = Math.max(1, this.activeDuration + deltaFrames);
            } 
            else if (this.mode === TimelineMode.ExtendingLeft) {
                // Moving left edge limits how far you can drag right (cannot exceed original end frame)
                const maxStart = this.activeStartFrame + this.activeDuration - 1;
                newStart = Math.max(1, Math.min(maxStart, this.activeStartFrame + deltaFrames));
                newDuration = (this.activeStartFrame + this.activeDuration) - newStart;
            }
            
            usePreferencesStore.getState().setGhostState({
                elementId: this.activeElementId, 
                originalStart: this.activeStartFrame, originalDuration: this.activeDuration, 
                newStart, newDuration
            });
        }
    }

    // --- The Commit (Rust Execution) ---
    private handleGlobalBlockUp(e: PointerEvent) {
        if (this.mode !== TimelineMode.Idle && this.mode !== TimelineMode.Scrubbing && this.activeElementId !== null) {
            const prefs = usePreferencesStore.getState();
            const engine = prefs.engineInstance;
            const ghost = prefs.ghostState;
            
            if (ghost && engine) {
                // If anything actually changed, fire the universal bounds update
                if (ghost.newStart !== ghost.originalStart || ghost.newDuration !== ghost.originalDuration) {
                    engine.update_exposure(this.activeElementId, ghost.originalStart, ghost.newStart, ghost.newDuration);
                    prefs.fetchTimelineState();
                }
            }
        }
        
        this.mode = TimelineMode.Idle;
        this.activeElementId = null;
        usePreferencesStore.getState().setGhostState(null);
        
        window.removeEventListener('pointermove', this.handleGlobalBlockMove);
        window.removeEventListener('pointerup', this.handleGlobalBlockUp);
    }
}