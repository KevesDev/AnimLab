import React, { useState, useEffect, useMemo } from 'react';
import { Layout, Model, TabNode } from 'flexlayout-react';
import 'flexlayout-react/style/dark.css';

import { defaultLayoutConfig } from './workspace/defaultLayout';
import { CanvasViewport } from './components/CanvasViewport';
import { PropertiesPanel } from './components/PropertiesPanel';
import { TimelinePanel } from './components/TimelinePanel';
import { LayerPropertiesPanel } from './components/LayerPropertiesPanel';
import { GlobalInputManager } from './engine_bridge/InputManager';
import { usePreferencesStore } from './store/PreferencesStore';
import { InputAction } from './store/shortcutStore';
import { ContextMenu } from './components/ContextMenu';
import { LayerModals } from './components/LayerModals'; // AAA FIX: Inject global modals

import init, { AnimLabEngine } from 'animlab-core';
import { MousePointer2, Scissors, Paintbrush, Pencil, Eraser, Layers } from 'lucide-react';

const Toolbar: React.FC = () => {
    const { activeTool, setActiveTool } = usePreferencesStore();
    const btnStyle = (t: InputAction) => ({
        padding: '6px', cursor: 'pointer', borderRadius: '4px', display: 'flex', alignItems: 'center', justifyContent: 'center',
        backgroundColor: activeTool === t ? '#4752c4' : 'transparent', color: activeTool === t ? '#fff' : '#888', marginTop: '6px', width: '32px', height: '32px'
    });
    return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '5px 0', height: '100%', backgroundColor: '#1c1d20' }}>
            <div style={btnStyle(InputAction.ToolSelect)} onClick={() => setActiveTool(InputAction.ToolSelect)}><MousePointer2 size={18} /></div>
            <div style={btnStyle(InputAction.ToolCutter)} onClick={() => setActiveTool(InputAction.ToolCutter)}><Scissors size={18} /></div>
            <div style={{ height: '1px', background: '#333', width: '20px', margin: '8px 0' }} />
            <div style={btnStyle(InputAction.ToolBrush)} onClick={() => setActiveTool(InputAction.ToolBrush)}><Paintbrush size={18} /></div>
            <div style={btnStyle(InputAction.ToolPencil)} onClick={() => setActiveTool(InputAction.ToolPencil)}><Pencil size={18} /></div>
            <div style={btnStyle(InputAction.ToolEraser)} onClick={() => setActiveTool(InputAction.ToolEraser)}><Eraser size={18} /></div>
        </div>
    );
};

const ArtLayerToolbar: React.FC = () => {
    const { activeArtLayer, setActiveArtLayer } = usePreferencesStore();
    return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '10px 0', gap: '6px', height: '100%', backgroundColor: '#1c1d20' }}>
            <Layers size={16} color="#444" style={{ marginBottom: '8px' }} />
            {['O', 'L', 'C', 'U'].map((l, i) => (
                <div key={l} onClick={() => setActiveArtLayer(i)} style={{ width: '24px', height: '24px', borderRadius: '4px', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '11px', fontWeight: 'bold', cursor: 'pointer', backgroundColor: activeArtLayer === i ? '#4752c4' : 'transparent', color: activeArtLayer === i ? '#fff' : '#888' }}>{l}</div>
            ))}
        </div>
    );
};

export const App: React.FC = () => {
    const layoutModel = useMemo(() => Model.fromJson(defaultLayoutConfig), []);

    useEffect(() => {
        const inputManager = GlobalInputManager.getInstance();
        
        const bootEngine = async () => {
            try {
                if (usePreferencesStore.getState().engineInstance) return;

                await init();
                const engine = new AnimLabEngine();
                const prefs = usePreferencesStore.getState();
                
                prefs.setEngineInstance(engine);
                engine.set_brush_settings(prefs.brush.thickness, prefs.brush.color[0], prefs.brush.color[1], prefs.brush.color[2], prefs.brush.color[3]);
                prefs.fetchTimelineState();
                
                console.info("[App] AnimLab WebAssembly Core securely initialized and injected.");
                inputManager.initialize();
            } catch (error) {
                console.error("[App] FATAL: Failed to initialize WebAssembly core:", error);
            }
        };

        bootEngine();
        return () => { inputManager.cleanup(); };
    }, []);

    const factory = (node: TabNode) => {
        const component = node.getComponent();
        switch (component) {
            case "CanvasNode": return <CanvasViewport />;
            case "PropertiesNode": return <PropertiesPanel />;
            case "LayerPropertiesNode": return <LayerPropertiesPanel />;
            case "TimelineNode": return <TimelinePanel />;
            case "ToolbarNode": return <Toolbar />;
            case "ArtLayerToolbarNode": return <ArtLayerToolbar />;
            default: return <div className="p-4 text-red-500">Missing Component: {component}</div>;
        }
    };

    return (
        <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', backgroundColor: '#111', overflow: 'hidden' }}>
            <div style={{ height: '32px', background: '#222327', borderBottom: '1px solid #111', display: 'flex', alignItems: 'center', padding: '0 12px', zIndex: 100 }}>
                <div style={{ color: '#aaa', fontSize: '10px', fontWeight: 'bold', letterSpacing: '1px' }}>ANIMLAB PRO</div>
                <div style={{ display: 'flex', gap: '15px', fontSize: '10px', color: '#666', marginLeft: '20px' }}>
                    <span>FILE</span><span>EDIT</span><span>VIEW</span>
                </div>
            </div>
            
            <div style={{ flex: 1, position: 'relative' }}>
                <Layout model={layoutModel} factory={factory} />
            </div>

            <ContextMenu />
            <LayerModals />

            <style>{`
                .flexlayout__tabset_header { background: #1c1d20 !important; border-bottom: 1px solid #111 !important; height: 22px !important; }
                .flexlayout__tab_button { padding: 0 10px !important; background: transparent !important; }
                .flexlayout__tab_button_content { font-size: 9px !important; color: #666 !important; text-transform: uppercase; font-weight: 700; letter-spacing: 0.5px; }
                .flexlayout__tab_button--selected { background: #2a2c30 !important; border-radius: 4px 4px 0 0 !important; }
                .flexlayout__tab_button--selected .flexlayout__tab_button_content { color: #ccc !important; }
                .flexlayout__tab_button_trailing { display: none !important; } 
                .flexlayout__tab { background: #141517 !important; overflow: hidden !important; }
                .flexlayout__splitter { background: #111 !important; width: 2px !important; }
            `}</style>
        </div>
    );
};

export default App;