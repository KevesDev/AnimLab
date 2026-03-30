import React, { useState, useEffect } from 'react';
import { Layout, Model, TabNode } from 'flexlayout-react';
import 'flexlayout-react/style/dark.css';

import { defaultLayoutConfig } from './workspace/defaultLayout';
import { CanvasViewport } from './components/CanvasViewport';
import { PropertiesPanel } from './components/PropertiesPanel';
import { GlobalInputManager } from './engine_bridge/InputManager';
import { usePreferencesStore } from './store/preferencesStore';
import { InputAction } from './store/shortcutStore';

// A sub-component for the toolbar to keep the factory function clean
const Toolbar: React.FC = () => {
    const activeTool = usePreferencesStore(state => state.activeTool);
    const setActiveTool = usePreferencesStore(state => state.setActiveTool);

    const getToolStyle = (tool: InputAction) => ({
        marginTop: '15px',
        padding: '8px',
        cursor: 'pointer',
        backgroundColor: activeTool === tool ? '#3a3d41' : 'transparent',
        borderRadius: '4px',
        border: activeTool === tool ? '1px solid #555' : '1px solid transparent',
        transition: 'all 0.1s ease-in-out'
    });

    return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '10px 0', color: '#888', userSelect: 'none' }}>
            <div style={{ ...getToolStyle(InputAction.ToolBrush), marginTop: 0 }} onClick={() => setActiveTool(InputAction.ToolBrush)} title="Brush (B)">🖌️</div>
            <div style={{ ...getToolStyle(InputAction.ToolPencil) }} onClick={() => setActiveTool(InputAction.ToolPencil)} title="Pencil (P)">✏️</div>
            <div style={{ ...getToolStyle(InputAction.ToolEraser) }} onClick={() => setActiveTool(InputAction.ToolEraser)} title="Eraser (E)">🧼</div>
            <div style={{ ...getToolStyle(InputAction.ToolLasso) }} onClick={() => setActiveTool(InputAction.ToolLasso)} title="Lasso (L)">✂️</div>
        </div>
    );
};

export const App: React.FC = () => {
    const [layoutModel] = useState(() => Model.fromJson(defaultLayoutConfig));

    // Initialize the AAA Input Routing Gatekeeper
    useEffect(() => {
        const inputManager = GlobalInputManager.getInstance();
        inputManager.initialize();

        return () => {
            inputManager.cleanup();
        };
    }, []);

    const factory = (node: TabNode) => {
        const component = node.getComponent();
        
        switch (component) {
            case "CanvasNode":
                return <CanvasViewport />;
            case "PropertiesNode":
                return <PropertiesPanel />;
            case "ToolbarNode":
                return <Toolbar />;
            default:
                return <div style={{ color: 'red', padding: '10px' }}>Unknown Layout Node: {component}</div>;
        }
    };

    return (
        <div style={{ position: 'absolute', top: 0, left: 0, right: 0, bottom: 0, overflow: 'hidden', backgroundColor: '#1e1e1e' }}>
            <Layout model={layoutModel} factory={factory} />
        </div>
    );
};

export default App;