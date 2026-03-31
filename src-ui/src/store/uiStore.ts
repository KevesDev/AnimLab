import { create } from 'zustand';

interface ContextMenuState {
    isOpen: boolean;
    x: number;
    y: number;
    hasSelection: boolean;
}

interface UIStoreState {
    contextMenu: ContextMenuState;
    openContextMenu: (x: number, y: number, hasSelection: boolean) => void;
    closeContextMenu: () => void;
}

export const useUIStore = create<UIStoreState>((set) => ({
    contextMenu: { isOpen: false, x: 0, y: 0, hasSelection: false },

    openContextMenu: (x, y, hasSelection) => set({
        contextMenu: { isOpen: true, x, y, hasSelection }
    }),

    closeContextMenu: () => set((state) => {
        if (!state.contextMenu.isOpen) return state;
        return { contextMenu: { ...state.contextMenu, isOpen: false } };
    })
}));