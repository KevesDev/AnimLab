import { create } from 'zustand';
import { InputAction } from './shortcutStore';

export interface BrushPreferences {
    thickness: number;
    color: [number, number, number, number];
    smoothing: number;
}

interface PreferencesState {
    brush: BrushPreferences;
    activeTool: InputAction;
    engineInstance: any | null;
    
    setBrushThickness: (thickness: number) => void;
    setBrushColor: (r: number, g: number, b: number, a: number) => void;
    setActiveTool: (tool: InputAction) => void;
    setEngineInstance: (engine: any) => void;
    syncPreferencesToEngine: () => void;
}

export const usePreferencesStore = create<PreferencesState>((set, get) => ({
    brush: {
        thickness: 12.0,
        color: [0.9, 0.9, 0.9, 1.0],
        smoothing: 0.5,
    },
    activeTool: InputAction.ToolBrush,
    engineInstance: null,

    setBrushThickness: (thickness) => {
        set((state) => ({ brush: { ...state.brush, thickness } }));
        get().syncPreferencesToEngine();
    },

    setBrushColor: (r, g, b, a) => {
        set((state) => ({ brush: { ...state.brush, color: [r, g, b, a] } }));
        get().syncPreferencesToEngine();
    },

    setActiveTool: (tool) => {
        set({ activeTool: tool });
        
        const { engineInstance } = get();
        if (engineInstance && typeof engineInstance.set_active_tool === 'function') {
            try {
                // AAA SAFETY: The WASM Boundary Type-Guard.
                // Rust &str requires a strictly allocated memory pointer. 
                // Passing undefined or null will instantly panic the JS/WASM glue code.
                if (typeof tool !== 'string' || !tool) {
                    console.warn(`[WASM Bridge] Blocked corrupted payload. Expected string, received:`, tool);
                    return;
                }
                
                engineInstance.set_active_tool(tool);
            } catch (error) {
                console.error(`[WASM Bridge] FATAL ERROR during Tool Swap execution:`, error);
            }
        }
    },

    setEngineInstance: (engineInstance) => {
        set({ engineInstance });
        get().syncPreferencesToEngine();
    },

    syncPreferencesToEngine: () => {
        const { engineInstance, brush } = get();
        if (engineInstance && typeof engineInstance.set_brush_settings === 'function') {
            try {
                engineInstance.set_brush_settings(
                    brush.thickness,
                    brush.color[0],
                    brush.color[1],
                    brush.color[2],
                    brush.color[3]
                );
            } catch (error) {
                console.error(`[WASM Bridge] FATAL ERROR during Settings Sync execution:`, error);
            }
        }
    }
}));