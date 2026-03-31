import React, { useEffect } from 'react';
import { useUIStore } from '../store/uiStore';
import { usePreferencesStore } from '../store/PreferencesStore';

export const ContextMenu: React.FC = () => {
    const { contextMenu, closeContextMenu } = useUIStore();
    const { engineInstance } = usePreferencesStore();

    useEffect(() => {
        if (!contextMenu.isOpen) return;
        window.addEventListener('resize', closeContextMenu);
        return () => window.removeEventListener('resize', closeContextMenu);
    }, [contextMenu.isOpen, closeContextMenu]);

    if (!contextMenu.isOpen) return null;

    const executeEngineCommand = (command: string, ...args: any[]) => {
        if (engineInstance && typeof engineInstance[command] === 'function') {
            try {
                engineInstance[command](...args);
            } catch (err) {
                console.error(`[ContextMenu] Failed to execute ${command}:`, err);
            }
        }
        closeContextMenu();
    };

    const stopPropagation = (e: React.PointerEvent | React.MouseEvent) => {
        e.stopPropagation();
    };

    return (
        <div
            style={{
                position: 'fixed', left: contextMenu.x, top: contextMenu.y,
                backgroundColor: '#2b2d31', border: '1px solid #1e1f22',
                boxShadow: '0 8px 16px rgba(0,0,0,0.6)', borderRadius: '6px',
                padding: '6px 0', zIndex: 9999, color: '#dbdee1', fontSize: '13px',
                display: 'flex', flexDirection: 'column', minWidth: '180px',
                userSelect: 'none', fontFamily: 'sans-serif'
            }}
            onPointerDown={stopPropagation}
            onContextMenu={stopPropagation}
        >
            {contextMenu.hasSelection ? (
                <>
                    <MenuButton onClick={() => executeEngineCommand('cut_selection')}>Cut</MenuButton>
                    <MenuButton onClick={() => executeEngineCommand('copy_selection')}>Copy</MenuButton>
                    <MenuButton onClick={() => executeEngineCommand('paste_clipboard')}>Paste</MenuButton>
                    <MenuButton onClick={() => executeEngineCommand('delete_selection')}>Delete</MenuButton>
                    <MenuSeparator />
                    <MenuButton onClick={() => executeEngineCommand('group_selection')}>Group</MenuButton>
                    <MenuButton onClick={() => executeEngineCommand('ungroup_selection')}>Ungroup</MenuButton>
                    <MenuSeparator />
                    <div style={{ padding: '4px 12px', fontSize: '11px', color: '#80848e', textTransform: 'uppercase', fontWeight: 600, letterSpacing: '0.5px', marginTop: '4px' }}>Transform</div>
                    <MenuButton onClick={() => executeEngineCommand('flip_selection', true, false)}>Flip Horizontal</MenuButton>
                    <MenuButton onClick={() => executeEngineCommand('flip_selection', false, true)}>Flip Vertical</MenuButton>
                </>
            ) : (
                <>
                    <MenuButton onClick={() => executeEngineCommand('select_all')}>Select All</MenuButton>
                    <MenuButton onClick={() => executeEngineCommand('paste_clipboard')}>Paste</MenuButton>
                </>
            )}
        </div>
    );
};

const MenuButton: React.FC<{ onClick: () => void, children: React.ReactNode }> = ({ onClick, children }) => (
    <button 
        onClick={onClick}
        style={{
            background: 'transparent', border: 'none', color: 'inherit', padding: '6px 16px',
            textAlign: 'left', cursor: 'pointer', width: '100%', outline: 'none', fontSize: '13px'
        }}
        onMouseEnter={(e) => e.currentTarget.style.backgroundColor = '#4752c4'}
        onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'transparent'}
    >
        {children}
    </button>
);

const MenuSeparator = () => (
    <div style={{ height: '1px', backgroundColor: '#1e1f22', margin: '4px 0' }} />
);