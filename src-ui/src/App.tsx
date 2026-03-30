import React, { useState } from 'react';
import { Layout, Model, TabNode } from 'flexlayout-react';
import 'flexlayout-react/style/dark.css';

import { defaultLayoutConfig } from './workspace/defaultLayout';
import { CanvasViewport } from './components/CanvasViewport';
import { PropertiesPanel } from './components/PropertiesPanel';

export const App: React.FC = () => {
    // AAA FIX: The Layout model MUST be bound to React's state lifecycle.
    // Using a static useRef causes the mathematical grid to detach from the DOM during drag events.
    const [layoutModel] = useState(() => Model.fromJson(defaultLayoutConfig));

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
        // The layout container must be absolutely anchored to the OS window bounds.
        // If it is static, FlexLayout's coordinate hit-testing calculates the drop zones incorrectly.
        <div style={{ position: 'absolute', top: 0, left: 0, right: 0, bottom: 0, overflow: 'hidden', backgroundColor: '#1e1e1e' }}>
            <Layout model={layoutModel} factory={factory} />
        </div>
    );
};

export default App;