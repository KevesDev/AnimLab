export const defaultLayoutConfig = {
    global: {
        rootOrientationVertical: true, // AAA FIX: This is the ONLY way to make the Root split Top/Bottom
        tabSetEnableTabStrip: true,
        tabSetHeaderHeight: 24,
        tabSetTabStripHeight: 24,
        splitterSize: 2,
        enableEdgeDock: false,
    },
    layout: {
        type: "row",
        weight: 100,
        children: [
            // Top Workspace: Since the Root is Vertical, this Row will be Horizontal
            {
                type: "row",
                weight: 80,
                children: [
                    {
                        type: "tabset",
                        width: 45,
                        enableTabStrip: false,
                        id: "tools-area",
                        children: [{ type: "tab", id: "tools", component: "ToolbarNode" }]
                    },
                    {
                        type: "tabset",
                        weight: 100, 
                        id: "canvas-area",
                        children: [{ type: "tab", id: "canvas", name: "Camera View", component: "CanvasNode", selected: true }]
                    },
                    {
                        type: "tabset",
                        width: 40,
                        enableTabStrip: false,
                        id: "art-layers-area",
                        children: [{ type: "tab", id: "art-layers", component: "ArtLayerToolbarNode" }]
                    },
                    {
                        type: "tabset",
                        width: 280, 
                        id: "properties-area",
                        children: [
                            { type: "tab", id: "properties", name: "Tool Properties", component: "PropertiesNode" },
                            { type: "tab", id: "layer-properties", name: "Layer Properties", component: "LayerPropertiesNode" }
                        ]
                    }
                ]
            },
            // Bottom Timeline: Direct child of a Vertical Row = Full-width bottom panel
            {
                type: "tabset",
                weight: 20, 
                id: "timeline-area",
                children: [{ type: "tab", id: "timeline", name: "Timeline", component: "TimelineNode" }]
            }
        ]
    }
};