import { create } from 'zustand';
import { InputAction } from './shortcutStore';

interface BrushSettings { thickness: number; color: [number, number, number, number]; smoothing: number; }
interface Modifiers { constrain: boolean; center: boolean; }
interface ModifierBindings { constrain: string; center: string; }

export interface TimelineLayer { id: bigint; name: string; }
export interface TimelineBlock { elementId: bigint; start: number; duration: number; id: bigint; }

// AAA FIX: Upgraded Ghost State handles both Start Frame and Duration changes simultaneously
export interface GhostState { elementId: bigint; originalStart: number; originalDuration: number; newStart: number; newDuration: number; }

interface PreferencesState {
    engineInstance: any | null;
    activeTool: InputAction;
    brush: BrushSettings;
    modifiers: Modifiers;
    modifierBindings: ModifierBindings;
    activeArtLayer: number; 
    currentFrame: number; 
    
    timelineLayers: TimelineLayer[];
    timelineBlocks: TimelineBlock[];
    ghostState: GhostState | null;
    selectedLayerId: bigint | null; // AAA FIX: Centralized Layer Selection

    setEngineInstance: (engine: any) => void;
    setActiveTool: (tool: InputAction) => void;
    setBrushThickness: (thickness: number) => void;
    setBrushColor: (r: number, g: number, b: number, a: number) => void;
    setBrushSmoothing: (smoothing: number) => void;
    setModifier: (key: keyof Modifiers, value: boolean) => void;
    
    setActiveArtLayer: (layerIndex: number) => void;
    setLayerOpacity: (elementId: number, opacity: number) => void;
    setLayerVisibility: (elementId: number, isVisible: boolean) => void;
    
    setCurrentFrame: (frame: number) => void; 
    setSelectedLayerId: (id: bigint | null) => void;
    setGhostState: (ghost: GhostState | null) => void;
    fetchTimelineState: () => void;
}

export const usePreferencesStore = create<PreferencesState>((set, get) => ({
    engineInstance: null,
    activeTool: InputAction.ToolBrush,
    brush: { thickness: 4.0, color: [0.08, 0.08, 0.08, 1.0], smoothing: 0.5 },
    modifiers: { constrain: false, center: false },
    modifierBindings: { constrain: 'Shift', center: 'Alt' },
    activeArtLayer: 1, 
    currentFrame: 1,
    timelineLayers: [], 
    timelineBlocks: [],
    ghostState: null,
    selectedLayerId: null,

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
    setBrushSmoothing: (smoothing) => { set((state) => ({ brush: { ...state.brush, smoothing } })); },
    setModifier: (key, value) => { set((state) => ({ modifiers: { ...state.modifiers, [key]: value } })); },
    
    setActiveArtLayer: (layerIndex) => {
        set({ activeArtLayer: layerIndex });
        const engine = get().engineInstance;
        if (engine && typeof engine.set_active_art_layer === 'function') { engine.set_active_art_layer(layerIndex); }
    },
    setLayerOpacity: (elementId, opacity) => {
        const engine = get().engineInstance;
        if (engine && typeof engine.set_layer_opacity === 'function') { engine.set_layer_opacity(BigInt(elementId), opacity); }
    },
    setLayerVisibility: (elementId, isVisible) => {
        const engine = get().engineInstance;
        if (engine && typeof engine.set_layer_visibility === 'function') { engine.set_layer_visibility(BigInt(elementId), isVisible); }
    },
    
    setCurrentFrame: (frame) => {
        set({ currentFrame: frame });
        const engine = get().engineInstance;
        if (engine && typeof engine.set_current_frame === 'function') { engine.set_current_frame(frame); }
    },
    
    setSelectedLayerId: (id) => set({ selectedLayerId: id }),
    setGhostState: (ghost) => set({ ghostState: ghost }),

    fetchTimelineState: () => {
        const engine = get().engineInstance;
        if (engine && typeof engine.get_timeline_blocks === 'function') {
            try {
                const layersJson = engine.get_timeline_layers();
                const layersData = JSON.parse(layersJson).map((l: any) => ({ ...l, id: BigInt(l.id) }));
                
                const rawBlocks = engine.get_timeline_blocks(); 
                const blocks: TimelineBlock[] = [];
                let i = 0;
                while (i < rawBlocks.length) {
                    const elementId = BigInt(rawBlocks[i++]);
                    const numBlocks = Number(rawBlocks[i++]);
                    for (let b = 0; b < numBlocks; b++) {
                        const start = Number(rawBlocks[i++]);
                        const duration = Number(rawBlocks[i++]);
                        const id = BigInt(rawBlocks[i++]);
                        blocks.push({ elementId, start, duration, id });
                    }
                }
                set({ timelineLayers: layersData, timelineBlocks: blocks });
            } catch (e) {
                console.error("[PreferencesStore] Failed to map timeline state from WASM", e);
            }
        }
    }
}));