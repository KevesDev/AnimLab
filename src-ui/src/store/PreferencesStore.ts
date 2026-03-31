import { create } from 'zustand';
import { InputAction } from './shortcutStore';

interface BrushSettings { thickness: number; color: [number, number, number, number]; smoothing: number; }
interface Modifiers { constrain: boolean; center: boolean; }
interface ModifierBindings { constrain: string; center: string; }

interface PreferencesState {
    engineInstance: any | null;
    activeTool: InputAction;
    brush: BrushSettings;
    modifiers: Modifiers;
    modifierBindings: ModifierBindings;
    activeArtLayer: number; // 0: Overlay, 1: LineArt, 2: ColorArt, 3: Underlay

    setEngineInstance: (engine: any) => void;
    setActiveTool: (tool: InputAction) => void;
    setBrushThickness: (thickness: number) => void;
    setBrushColor: (r: number, g: number, b: number, a: number) => void;
    setBrushSmoothing: (smoothing: number) => void;
    setModifier: (key: keyof Modifiers, value: boolean) => void;
    
    // AAA ARCHITECTURE: Compositor Integration Hooks
    setActiveArtLayer: (layerIndex: number) => void;
    setLayerOpacity: (elementId: number, opacity: number) => void;
    setLayerVisibility: (elementId: number, isVisible: boolean) => void;
}

export const usePreferencesStore = create<PreferencesState>((set, get) => ({
    engineInstance: null,
    activeTool: InputAction.ToolBrush,
    brush: { thickness: 10.0, color: [1.0, 1.0, 1.0, 1.0], smoothing: 0.5 },
    modifiers: { constrain: false, center: false },
    modifierBindings: { constrain: 'Shift', center: 'Alt' },
    activeArtLayer: 1, // Default to Line Art

    setEngineInstance: (engine) => set({ engineInstance: engine }),
    
    setActiveTool: (tool) => {
        set({ activeTool: tool });
        const engine = get().engineInstance;
        if (engine && typeof engine.set_active_tool === 'function') { engine.set_active_tool(tool); }
    },
    setBrushThickness: (thickness) => {
        set((state) => ({ brush: { ...state.brush, thickness } }));
        const engine = get().engineInstance;
        const b = get().brush;
        if (engine && typeof engine.set_brush_settings === 'function') { engine.set_brush_settings(thickness, b.color[0], b.color[1], b.color[2], b.color[3]); }
    },
    setBrushColor: (r, g, b, a) => {
        set((state) => ({ brush: { ...state.brush, color: [r, g, b, a] } }));
        const engine = get().engineInstance;
        const thickness = get().brush.thickness;
        if (engine && typeof engine.set_brush_settings === 'function') { engine.set_brush_settings(thickness, r, g, b, a); }
    },
    setBrushSmoothing: (smoothing) => {
        set((state) => ({ brush: { ...state.brush, smoothing } }));
    },
    setModifier: (key, value) => {
        set((state) => ({ modifiers: { ...state.modifiers, [key]: value } }));
    },
    
    setActiveArtLayer: (layerIndex) => {
        set({ activeArtLayer: layerIndex });
        const engine = get().engineInstance;
        if (engine && typeof engine.set_active_art_layer === 'function') {
            engine.set_active_art_layer(layerIndex);
        }
    },
    setLayerOpacity: (elementId, opacity) => {
        const engine = get().engineInstance;
        if (engine && typeof engine.set_layer_opacity === 'function') {
            engine.set_layer_opacity(elementId, opacity);
        }
    },
    setLayerVisibility: (elementId, isVisible) => {
        const engine = get().engineInstance;
        if (engine && typeof engine.set_layer_visibility === 'function') {
            engine.set_layer_visibility(elementId, isVisible);
        }
    }
}));