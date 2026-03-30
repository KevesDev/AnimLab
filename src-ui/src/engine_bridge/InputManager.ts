import { useShortcutStore, InputAction } from '../store/shortcutStore';
import { usePreferencesStore } from '../store/preferencesStore';

export class GlobalInputManager {
    private static instance: GlobalInputManager;
    private isInitialized = false;

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
            window.removeEventListener('contextmenu', this.preventContextMenu);
            this.isInitialized = false;
            console.info("[InputManager] Core Event Listeners safely unmounted.");
        } catch (error) {
            console.error("[InputManager] ERROR: Memory leak risk. Failed to unmount listeners.", error);
        }
    }

    private preventContextMenu = (e: MouseEvent) => {
        e.preventDefault();
    };

    private handleKeyDown = (e: KeyboardEvent) => {
        try {
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
                return;
            }

            const chordParts: string[] = [];
            if (e.ctrlKey) chordParts.push('Control');
            if (e.metaKey) chordParts.push('Meta');
            if (e.altKey) chordParts.push('Alt');
            if (e.shiftKey) chordParts.push('Shift');
            
            if (!['ControlLeft', 'ControlRight', 'ShiftLeft', 'ShiftRight', 'AltLeft', 'AltRight', 'MetaLeft', 'MetaRight'].includes(e.code)) {
                chordParts.push(e.code); 
            } else {
                return; 
            }

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

    private dispatchAction(action: InputAction) {
        const prefs = usePreferencesStore.getState();
        const engine = prefs.engineInstance;

        try {
            switch (action) {
                case InputAction.Undo:
                    console.info("[InputManager] Dispatching Semantic Action: Undo");
                    // AAA FIX: Route Semantic UI action to the compiled Rust History API
                    if (engine) engine.trigger_undo();
                    break;
                case InputAction.Redo:
                    console.info("[InputManager] Dispatching Semantic Action: Redo");
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
                    console.info(`[InputManager] State Mutated: Active Tool is now ${action}`);
                    break;
                    
                default:
                    console.warn(`[InputManager] Semantic Action '${action}' lacks execution logic in the dispatch router.`);
                    break;
            }
        } catch (error) {
            console.error(`[InputManager] FATAL: The Rust Engine or React UI threw an unhandled exception while executing '${action}'.`, error);
        }
    }
}