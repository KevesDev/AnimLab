import { useShortcutStore, InputAction } from '../store/shortcutStore';
import { usePreferencesStore } from '../store/PreferencesStore';

export class GlobalInputManager {
    private static instance: GlobalInputManager;
    private isInitialized = false;
    private activeCanvas: HTMLCanvasElement | null = null;

    private constructor() {}

    public static getInstance(): GlobalInputManager {
        if (!GlobalInputManager.instance) {
            GlobalInputManager.instance = new GlobalInputManager();
        }
        return GlobalInputManager.instance;
    }

    public initialize() {
        if (this.isInitialized) return;
        try {
            window.addEventListener('keydown', this.handleKeyDown);
            window.addEventListener('keyup', this.handleKeyUp);
            window.addEventListener('contextmenu', this.preventContextMenu); 
            this.isInitialized = true;
            console.info("[InputManager] Core Event Listeners securely mounted.");
        } catch (error) {
            console.error("[InputManager] FATAL: Failed to mount input listeners.", error);
        }
    }

    public cleanup() {
        if (!this.isInitialized) return;
        try {
            window.removeEventListener('keydown', this.handleKeyDown);
            window.removeEventListener('keyup', this.handleKeyUp);
            window.removeEventListener('contextmenu', this.preventContextMenu);
            this.detachCanvas();
            this.isInitialized = false;
            console.info("[InputManager] Core Event Listeners safely unmounted.");
        } catch (error) {
            console.error("[InputManager] ERROR: Memory leak risk. Failed to unmount listeners.", error);
        }
    }

    // AAA ARCHITECTURE: Native DOM Pointer Binding (Bypassing React entirely)
    public attachCanvas(canvas: HTMLCanvasElement) {
        if (this.activeCanvas) this.detachCanvas();
        this.activeCanvas = canvas;
        this.activeCanvas.addEventListener('pointerdown', this.handlePointerDown);
        this.activeCanvas.addEventListener('pointermove', this.handlePointerMove);
        this.activeCanvas.addEventListener('pointerup', this.handlePointerUp);
        this.activeCanvas.addEventListener('pointercancel', this.handlePointerUp);
        this.activeCanvas.addEventListener('pointerleave', this.handlePointerUp);
        console.info("[InputManager] Native Pointer Events securely attached to Canvas.");
    }

    public detachCanvas() {
        if (!this.activeCanvas) return;
        this.activeCanvas.removeEventListener('pointerdown', this.handlePointerDown);
        this.activeCanvas.removeEventListener('pointermove', this.handlePointerMove);
        this.activeCanvas.removeEventListener('pointerup', this.handlePointerUp);
        this.activeCanvas.removeEventListener('pointercancel', this.handlePointerUp);
        this.activeCanvas.removeEventListener('pointerleave', this.handlePointerUp);
        this.activeCanvas = null;
        console.info("[InputManager] Native Pointer Events detached from Canvas.");
    }

    private preventContextMenu = (e: MouseEvent) => {
        e.preventDefault();
    };

    // --- HARDWARE POINTER ROUTING ---
    private handlePointerDown = (e: PointerEvent) => {
        if (!this.activeCanvas) return;
        const engine = usePreferencesStore.getState().engineInstance;
        if (!engine) return;

        try {
            this.activeCanvas.setPointerCapture(e.pointerId);
            
            const rect = this.activeCanvas.getBoundingClientRect();
            const dpr = window.devicePixelRatio || 1;
            const x = (e.clientX - rect.left) * dpr;
            const y = (e.clientY - rect.top) * dpr;
            const pressure = e.pointerType === 'pen' && e.pressure !== 0 ? e.pressure : 1.0;
            
            const { constrain, center } = usePreferencesStore.getState().modifiers;
            engine.begin_stroke(x, y, pressure, constrain, center);
        } catch (err) {
            console.error("[InputManager] Pipeline crashed during pointer down:", err);
        }
    };

    private handlePointerMove = (e: PointerEvent) => {
        if (!this.activeCanvas) return;
        const engine = usePreferencesStore.getState().engineInstance;
        if (!engine) return;

        try {
            const rect = this.activeCanvas.getBoundingClientRect();
            const dpr = window.devicePixelRatio || 1;
            const x = (e.clientX - rect.left) * dpr;
            const y = (e.clientY - rect.top) * dpr;
            const pressure = e.pointerType === 'pen' && e.pressure !== 0 ? e.pressure : 1.0;
            
            const { constrain, center } = usePreferencesStore.getState().modifiers;

            if (e.buttons === 0) {
                engine.hover(x, y, constrain, center);
            } else if (this.activeCanvas.hasPointerCapture(e.pointerId)) {
                engine.push_point(x, y, pressure, constrain, center);
            }
        } catch (err) {
            console.error("[InputManager] Pipeline crashed during pointer move:", err);
        }
    };

    private handlePointerUp = (e: PointerEvent) => {
        if (!this.activeCanvas) return;
        const engine = usePreferencesStore.getState().engineInstance;
        if (!engine) return;

        try {
            if (this.activeCanvas.hasPointerCapture(e.pointerId)) {
                this.activeCanvas.releasePointerCapture(e.pointerId);
            }
            engine.end_stroke();
        } catch (err) {
            console.error("[InputManager] Pipeline crashed during pointer up:", err);
        }
    };

    // --- KEYBOARD ROUTING ---
    private handleKeyDown = (e: KeyboardEvent) => {
        try {
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

            const prefs = usePreferencesStore.getState();
            if (e.key === prefs.modifierBindings.constrain) prefs.setModifier('constrain', true);
            if (e.key === prefs.modifierBindings.center) prefs.setModifier('center', true);

            const chordParts: string[] = [];
            if (e.ctrlKey) chordParts.push('Control');
            if (e.metaKey) chordParts.push('Meta');
            if (e.altKey) chordParts.push('Alt');
            if (e.shiftKey) chordParts.push('Shift');
            
            if (!['ControlLeft', 'ControlRight', 'ShiftLeft', 'ShiftRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(e.code)) {
                chordParts.push(e.code); 
            } else { return; }

            const chord = chordParts.join('+');
            const action = useShortcutStore.getState().keyMap[chord];

            if (action) {
                e.preventDefault(); 
                this.dispatchAction(action);
            }
        } catch (error) {
            console.error(`[InputManager] Pipeline crashed while parsing hardware event: ${e.code}`, error);
        }
    };

    private handleKeyUp = (e: KeyboardEvent) => {
        try {
            const prefs = usePreferencesStore.getState();
            if (e.key === prefs.modifierBindings.constrain) prefs.setModifier('constrain', false);
            if (e.key === prefs.modifierBindings.center) prefs.setModifier('center', false);
        } catch (error) {
            console.error(`[InputManager] Failed tracking KeyUp state.`, error);
        }
    };

    private dispatchAction(action: InputAction) {
        const prefs = usePreferencesStore.getState();
        const engine = prefs.engineInstance;

        try {
            switch (action) {
                case InputAction.Undo:
                    if (engine) engine.trigger_undo();
                    break;
                case InputAction.Redo:
                    if (engine) engine.trigger_redo();
                    break;
                case InputAction.DecreaseBrushSize:
                    prefs.setBrushThickness(Math.max(1.0, prefs.brush.thickness - 1.0));
                    break;
                case InputAction.IncreaseBrushSize:
                    prefs.setBrushThickness(Math.min(100.0, prefs.brush.thickness + 1.0));
                    break;
                case InputAction.ToolBrush:
                case InputAction.ToolPencil:
                case InputAction.ToolEraser:
                case InputAction.ToolSelect:
                case InputAction.ToolCutter:
                case InputAction.ToolContourEditor:
                case InputAction.ToolCenterlineEditor:
                case InputAction.ToolPerspective:
                case InputAction.ToolPaint:
                case InputAction.ToolPaintUnpainted:
                case InputAction.ToolUnpaint:
                case InputAction.ToolCloseGap:
                case InputAction.ToolDropper:
                case InputAction.ToolLine:
                case InputAction.ToolRectangle:
                case InputAction.ToolEllipse:
                case InputAction.ToolPolyline:
                case InputAction.ToolText:
                case InputAction.ToolPivot:
                case InputAction.ToolMorphing:
                case InputAction.ToolRigging:
                case InputAction.ToolHand:
                case InputAction.ToolZoom:
                case InputAction.ToolRotateView:
                    prefs.setActiveTool(action);
                    break;
                default:
                    console.warn(`[InputManager] Semantic Action '${action}' lacks execution logic.`);
                    break;
            }
        } catch (error) {
            console.error(`[InputManager] FATAL Exception executing '${action}'.`, error);
        }
    }
}