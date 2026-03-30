import React, { useRef } from 'react';
import { Layout, Model, TabNode } from 'flexlayout-react';
import 'flexlayout-react/style/dark.css';

import { defaultLayoutConfig } from './workspace/defaultLayout';
import { CanvasViewport } from './components/CanvasViewport';
import { PropertiesPanel } from './components/PropertiesPanel';

export const App: React.FC = () => {
    // Initialize the FlexLayout mathematical model
    const layoutModel = useRef(Model.fromJson(defaultLayoutConfig));

    // The factory maps the string names in defaultLayout.ts to actual React components
    const factory = (node: TabNode) => {
        const component = node.getComponent();
        
        switch (component) {
            case "CanvasNode":
                return <CanvasViewport />;
            case "PropertiesNode":
                return <PropertiesPanel />;
            case "ToolbarNode":
                return (
                    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '10px 0', color: '#888' }}>
                        <div>🖌️</div>
                        <div style={{ marginTop: '15px' }}>✏️</div>
                        <div style={{ marginTop: '15px' }}>✂️</div>
                    </div>
                );
            default:
                return <div style={{ color: 'red', padding: '10px' }}>Unknown Layout Node: {component}</div>;
        }
    };

    return (
        <div style={{ width: '100vw', height: '100vh', overflow: 'hidden', backgroundColor: '#1e1e1e' }}>
            <Layout model={layoutModel.current} factory={factory} />
        </div>
    );
};

export default App;