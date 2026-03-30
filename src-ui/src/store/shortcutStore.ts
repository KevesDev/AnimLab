import { create } from 'zustand';

// AAA ARCHITECTURE: Semantic Actions
// This exhaustive list serves as the master contract between the React UI and the Rust Engine.
// If a tool exists here, the engine must eventually know how to mathematically process it.
export enum InputAction {
    // System & File Actions
    Undo = 'Undo',
    Redo = 'Redo',
    SaveProject = 'SaveProject',
    
    // Playback & Timeline
    PlayPause = 'PlayPause',
    NextFrame = 'NextFrame',
    PrevFrame = 'PrevFrame',

    // Primary Drawing Tools
    ToolBrush = 'ToolBrush',
    ToolPencil = 'ToolPencil',
    ToolEraser = 'ToolEraser',
    
    // Selection & Editing Tools
    ToolSelect = 'ToolSelect',                   // Black Arrow (Move/Scale/Rotate entire strokes)
    ToolCutter = 'ToolCutter',                   // Lasso Cut
    ToolContourEditor = 'ToolContourEditor',     // White Arrow (Edit bezier vertices directly)
    ToolCenterlineEditor = 'ToolCenterlineEditor', // Tweak pencil thickness/spines dynamically
    ToolPerspective = 'ToolPerspective',         // 4-point quad deformation

    // Color & Paint Tools
    ToolPaint = 'ToolPaint',                     // Standard Paint Bucket
    ToolPaintUnpainted = 'ToolPaintUnpainted',   // Only fills transparent areas (protects existing color)
    ToolUnpaint = 'ToolUnpaint',                 // Removes color from filled zones
    ToolCloseGap = 'ToolCloseGap',               // Draws an invisible stroke to seal leaking fills
    ToolDropper = 'ToolDropper',                 // Color Picker

    // Geometric & Text Tools
    ToolLine = 'ToolLine',
    ToolRectangle = 'ToolRectangle',
    ToolEllipse = 'ToolEllipse',
    ToolPolyline = 'ToolPolyline',
    ToolText = 'ToolText',

    // Rigging & Animation Tools
    ToolPivot = 'ToolPivot',                     // Set rotational anchor for pegs
    ToolMorphing = 'ToolMorphing',               // Generate morph hints between keyframes
    ToolRigging = 'ToolRigging',                 // Bone/Envelope mesh setup

    // Viewport Navigation
    ToolHand = 'ToolHand',                       // Pan canvas
    ToolZoom = 'ToolZoom',                       // Zoom canvas
    ToolRotateView = 'ToolRotateView',           // Rotate canvas (Virtual Animator's Disk)

    // Modifiers
    DecreaseBrushSize = 'DecreaseBrushSize',
    IncreaseBrushSize = 'IncreaseBrushSize',
}

interface ShortcutState {
    keyMap: Record<string, InputAction>;
    bindKey: (chord: string, action: InputAction) => void;
}

export const useShortcutStore = create<ShortcutState>((set) => ({
    // Default Industry-Standard Keybinds (Mapped to Toon Boom / Clip Studio standards)
    keyMap: {
        'Control+KeyZ': InputAction.Undo,
        'Meta+KeyZ': InputAction.Undo, 
        
        'Control+KeyY': InputAction.Redo,
        'Meta+KeyY': InputAction.Redo, 
        'Control+Shift+KeyZ': InputAction.Redo,
        'Meta+Shift+KeyZ': InputAction.Redo,
        
        'Control+KeyS': InputAction.SaveProject,
        'Meta+KeyS': InputAction.SaveProject,

        'KeyB': InputAction.ToolBrush,
        'KeyP': InputAction.ToolPencil,
        'KeyE': InputAction.ToolEraser,
        'KeyL': InputAction.ToolCutter,
        'KeyS': InputAction.ToolSelect,
        'KeyA': InputAction.ToolContourEditor,
        'KeyI': InputAction.ToolDropper,
        'KeyF': InputAction.ToolPaint,
        'KeyK': InputAction.ToolCloseGap,
        
        'Space': InputAction.ToolHand,
        'KeyZ': InputAction.ToolZoom,
        
        'BracketLeft': InputAction.DecreaseBrushSize,
        'BracketRight': InputAction.IncreaseBrushSize,
        
        'Comma': InputAction.PrevFrame,
        'Period': InputAction.NextFrame,
    },

    bindKey: (chord, action) => {
        try {
            set((state) => ({
                keyMap: { ...state.keyMap, [chord]: action }
            }));
            console.info(`[ShortcutStore] Successfully bound ${chord} to ${action}`);
        } catch (error) {
            console.error(`[ShortcutStore] FATAL: Failed to bind key ${chord}. Data corruption likely.`, error);
        }
    }
}));