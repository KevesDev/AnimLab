import { create } from 'zustand';
import { InputAction } from './shortcutStore';

export interface BrushPreferences {
    thickness: number;
    color: [number, number, number, number];
    smoothing: number;
}

interface PreferencesState {
    // Global User Preferences
    brush: BrushPreferences;
    activeTool: InputAction;
    
    // The active memory pointer to the Rust WASM Engine
    engineInstance: any | null;
    
    // Actions
    setBrushThickness: (thickness: number) => void;
    setBrushColor: (r: number, g: number, b: number, a: number) => void;
    setActiveTool: (tool: InputAction) => void;
    setEngineInstance: (engine: any) => void;
    
    // Pushes the current React state across the WASM bridge into Rust memory
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
        // Future: This is where we will call engine.set_active_tool(tool) in Rust
    },

    setEngineInstance: (engineInstance) => {
        set({ engineInstance });
        get().syncPreferencesToEngine();
    },

    syncPreferencesToEngine: () => {
        const { engineInstance, brush } = get();
        if (engineInstance) {
            engineInstance.set_brush_settings(
                brush.thickness,
                brush.color[0],
                brush.color[1],
                brush.color[2],
                brush.color[3]
            );
        }
    }
}));