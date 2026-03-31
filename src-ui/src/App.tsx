import React, { useState, useEffect } from 'react';
import { Layout, Model, TabNode } from 'flexlayout-react';
import 'flexlayout-react/style/dark.css';

import { defaultLayoutConfig } from './workspace/defaultLayout';
import { CanvasViewport } from './components/CanvasViewport';
import { PropertiesPanel } from './components/PropertiesPanel';
import { GlobalInputManager } from './engine_bridge/InputManager';
import { usePreferencesStore } from './store/PreferencesStore';
import { InputAction } from './store/shortcutStore';
import { ContextMenu } from './components/ContextMenu'; // AAA: Imported Context Menu

// AAA UI: Industry-Standard Monochrome SVG Icons
import {
    MousePointer2, Scissors, PenTool, Activity, BoxSelect,
    Paintbrush, Pencil, Eraser, PaintBucket, Wand2, Slash,
    Magnet, Pipette, Minus, Square, Circle, Share2, Type,
    Crosshair, Infinity as InfinityIcon, Bone, Hand, Search, RotateCw
} from 'lucide-react';

const Toolbar: React.FC = () => {
    const activeTool = usePreferencesStore(state => state.activeTool);
    const setActiveTool = usePreferencesStore(state => state.setActiveTool);

    const getToolStyle = (tool: InputAction) => ({
        marginTop: '6px',
        padding: '6px',
        cursor: 'pointer',
        backgroundColor: activeTool === tool ? '#3a3d41' : 'transparent',
        borderRadius: '4px',
        border: activeTool === tool ? '1px solid #555' : '1px solid transparent',
        transition: 'all 0.1s ease-in-out',
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        width: '32px',
        height: '32px',
        boxSizing: 'border-box' as const,
        color: activeTool === tool ? '#ffffff' : '#888888',
    });

    const ToolButton: React.FC<{ tool: InputAction; icon: React.ElementType; title: string }> = ({ tool, icon: Icon, title }) => (
        <div style={getToolStyle(tool)} onClick={() => setActiveTool(tool)} title={title}>
            <Icon size={18} strokeWidth={activeTool === tool ? 2.2 : 1.5} />
        </div>
    );

    const CategoryDivider: React.FC<{ title: string }> = ({ title }) => (
        <div style={{ fontSize: '9px', fontWeight: 'bold', marginTop: '12px', marginBottom: '4px', borderBottom: '1px solid #333', width: '80%', textAlign: 'center', paddingBottom: '2px', color: '#555' }}>
            {title}
        </div>
    );

    return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '10px 0', color: '#888', userSelect: 'none', height: '100%', overflowY: 'auto' }}>
            
            <CategoryDivider title="EDIT" />
            <ToolButton tool={InputAction.ToolSelect} icon={MousePointer2} title="Select (S)" />
            <ToolButton tool={InputAction.ToolCutter} icon={Scissors} title="Cutter (L)" />
            <ToolButton tool={InputAction.ToolContourEditor} icon={PenTool} title="Contour Editor (A)" />
            <ToolButton tool={InputAction.ToolCenterlineEditor} icon={Activity} title="Centerline Editor" />
            <ToolButton tool={InputAction.ToolPerspective} icon={BoxSelect} title="Perspective Tool" />

            <CategoryDivider title="DRAW" />
            <ToolButton tool={InputAction.ToolBrush} icon={Paintbrush} title="Vector Brush (B)" />
            <ToolButton tool={InputAction.ToolPencil} icon={Pencil} title="Vector Pencil (P)" />
            <ToolButton tool={InputAction.ToolEraser} icon={Eraser} title="Eraser (E)" />

            <CategoryDivider title="PAINT" />
            <ToolButton tool={InputAction.ToolPaint} icon={PaintBucket} title="Paint (F)" />
            <ToolButton tool={InputAction.ToolPaintUnpainted} icon={Wand2} title="Paint Unpainted" />
            <ToolButton tool={InputAction.ToolUnpaint} icon={Slash} title="Unpaint" />
            <ToolButton tool={InputAction.ToolCloseGap} icon={Magnet} title="Close Gap (K)" />
            <ToolButton tool={InputAction.ToolDropper} icon={Pipette} title="Dropper (I)" />

            <CategoryDivider title="SHAPES" />
            <ToolButton tool={InputAction.ToolLine} icon={Minus} title="Line" />
            <ToolButton tool={InputAction.ToolRectangle} icon={Square} title="Rectangle" />
            <ToolButton tool={InputAction.ToolEllipse} icon={Circle} title="Ellipse" />
            <ToolButton tool={InputAction.ToolPolyline} icon={Share2} title="Polyline" />
            <ToolButton tool={InputAction.ToolText} icon={Type} title="Text" />

            <CategoryDivider title="RIGGING" />
            <ToolButton tool={InputAction.ToolPivot} icon={Crosshair} title="Set Pivot" />
            <ToolButton tool={InputAction.ToolMorphing} icon={InfinityIcon} title="Morphing" />
            <ToolButton tool={InputAction.ToolRigging} icon={Bone} title="Rigging" />

            <CategoryDivider title="VIEW" />
            <ToolButton tool={InputAction.ToolHand} icon={Hand} title="Hand (Space)" />
            <ToolButton tool={InputAction.ToolZoom} icon={Search} title="Zoom (Z)" />
            <ToolButton tool={InputAction.ToolRotateView} icon={RotateCw} title="Rotate View" />
            
        </div>
    );
};

export const App: React.FC = () => {
    const [layoutModel] = useState(() => Model.fromJson(defaultLayoutConfig));

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
            <ContextMenu /> {/* AAA: Context Menu floats freely above flexlayout */}
        </div>
    );
};

export default App;