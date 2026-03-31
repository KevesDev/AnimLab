import React, { useState, useEffect, useMemo } from 'react';
import { Layout, Model, TabNode } from 'flexlayout-react';
import 'flexlayout-react/style/dark.css';

import { defaultLayoutConfig } from './workspace/defaultLayout';
import { CanvasViewport } from './components/CanvasViewport';
import { PropertiesPanel } from './components/PropertiesPanel';
import { TimelinePanel } from './components/TimelinePanel';
import { LayerPropertiesPanel } from './components/LayerPropertiesPanel';
import { GlobalInputManager } from './engine_bridge/InputManager';
import { RefreshCcw } from 'lucide-react';

export const App: React.FC = () => {
    // AAA FIX: layoutKey forces a hard remount to bypass internal library state persistence
    const [layoutKey, setLayoutKey] = useState(0);
    const layoutModel = useMemo(() => Model.fromJson(defaultLayoutConfig), [layoutKey]);

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
            case "ToolbarNode": return <div className="h-full bg-[#1c1d20] border-r border-black" />;
            case "ArtLayerToolbarNode": return <div className="h-full bg-[#1c1d20] border-l border-black" />;
            default: return <div>Missing Component: {component}</div>;
        }
    };

    const handleHardReset = () => {
        setLayoutKey(k => k + 1); // Triggers re-memoization of Model.fromJson
    };

    return (
        <div style={{ position: 'absolute', inset: 0, display: 'flex', flexDirection: 'column', backgroundColor: '#111', overflow: 'hidden' }}>
            <div style={{ height: '32px', background: '#222327', borderBottom: '1px solid #111', display: 'flex', alignItems: 'center', padding: '0 12px', justifyContent: 'space-between', zIndex: 100 }}>
                <div style={{ color: '#aaa', fontSize: '10px', fontWeight: 'bold', letterSpacing: '1px' }}>ANIMLAB PRO</div>
                <button 
                    onClick={handleHardReset}
                    style={{ background: '#333', color: '#888', border: 'none', borderRadius: '2px', fontSize: '9px', padding: '4px 8px', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '6px' }}
                >
                    <RefreshCcw size={10} /> RESET DOCKING
                </button>
            </div>
            <div style={{ flex: 1, position: 'relative' }}>
                <Layout key={layoutKey} model={layoutModel} factory={factory} />
            </div>

            <style>{`
                .flexlayout__tabset_header { background: #1c1d20 !important; color: #888 !important; border-bottom: 1px solid #000 !important; height: 24px !important; }
                .flexlayout__tab { background: #141517 !important; overflow: hidden !important; }
                .flexlayout__splitter { background: #111 !important; width: 2px !important; }
            `}</style>
        </div>
    );
};