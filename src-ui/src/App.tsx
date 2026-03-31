import React, { useState, useEffect } from 'react';
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

import {
    MousePointer2, Scissors, Paintbrush, Pencil, Eraser, Layers
} from 'lucide-react';

const Toolbar: React.FC = () => {
    const activeTool = usePreferencesStore(state => state.activeTool);
    const setActiveTool = usePreferencesStore(state => state.setActiveTool);

    const ToolButton: React.FC<{ tool: InputAction; icon: React.ElementType; title: string }> = ({ tool, icon: Icon, title }) => (
        <div 
            onClick={() => setActiveTool(tool)} title={title}
            style={{
                marginTop: '6px', padding: '6px', cursor: 'pointer',
                backgroundColor: activeTool === tool ? '#4752c4' : 'transparent',
                borderRadius: '4px', display: 'flex', justifyContent: 'center', alignItems: 'center',
                width: '32px', height: '32px', color: activeTool === tool ? '#fff' : '#888'
            }}
        >
            <Icon size={18} />
        </div>
    );

    return (
        <div className="hide-scrollbar" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '10px 0', height: '100%', overflow: 'hidden' }}>
            <ToolButton tool={InputAction.ToolSelect} icon={MousePointer2} title="Select" />
            <ToolButton tool={InputAction.ToolCutter} icon={Scissors} title="Cutter" />
            <div style={{ height: '1px', background: '#333', width: '20px', margin: '10px 0' }} />
            <ToolButton tool={InputAction.ToolBrush} icon={Paintbrush} title="Brush" />
            <ToolButton tool={InputAction.ToolPencil} icon={Pencil} title="Pencil" />
            <ToolButton tool={InputAction.ToolEraser} icon={Eraser} title="Eraser" />
        </div>
    );
};

const ArtLayerToolbar: React.FC = () => {
    const { activeArtLayer, setActiveArtLayer } = usePreferencesStore();
    return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '10px 0', gap: '6px' }}>
            <Layers size={16} color="#555" style={{ marginBottom: '4px' }} />
            {['O', 'L', 'C', 'U'].map((l, i) => (
                <div 
                    key={l} onClick={() => setActiveArtLayer(i)}
                    style={{
                        width: '28px', height: '28px', borderRadius: '4px', display: 'flex', alignItems: 'center', justifyContent: 'center',
                        fontSize: '12px', fontWeight: 'bold', cursor: 'pointer',
                        backgroundColor: activeArtLayer === i ? '#4752c4' : 'transparent',
                        color: activeArtLayer === i ? '#fff' : '#888'
                    }}
                >{l}</div>
            ))}
        </div>
    );
};

export const App: React.FC = () => {
    const [layoutModel] = useState(() => Model.fromJson(defaultLayoutConfig));

    useEffect(() => {
        const inputManager = GlobalInputManager.getInstance();
        inputManager.initialize();
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
            default: return <div>Unknown Node</div>;
        }
    };

    return (
        <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', backgroundColor: '#111', overflow: 'hidden' }}>
            {/* AAA TOP BAR: Full width Harmony Menu Bar */}
            <div style={{ height: '32px', background: '#222327', borderBottom: '1px solid #111', display: 'flex', alignItems: 'center', padding: '0 12px', gap: '20px', zIndex: 100 }}>
                <div style={{ color: '#aaa', fontSize: '10px', fontWeight: 'bold' }}>ANIMLAB PRO</div>
                <div style={{ display: 'flex', gap: '15px', fontSize: '10px', color: '#666' }}>
                    <span>FILE</span><span>EDIT</span><span>SCENE</span><span>VIEW</span>
                </div>
            </div>
            
            <div style={{ flex: 1, position: 'relative' }}>
                <Layout model={layoutModel} factory={factory} />
            </div>

            <style>{`
                .hide-scrollbar::-webkit-scrollbar { display: none; }
                .flexlayout__tabset_header { background: #1c1d20 !important; color: #888 !important; border-bottom: 1px solid #111 !important; }
                .flexlayout__tab { background: #141517 !important; overflow: hidden !important; }
                .flexlayout__splitter { background: #111 !important; }
            `}</style>
        </div>
    );
};

export default App;