export const defaultLayoutConfig = {
    global: {
        rootOrientationVertical: true,
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
            {
                type: "row",
                weight: 80, // Top section takes 80% of vertical height
                children: [
                    {
                        type: "tabset",
                        weight: 3, // AAA: Very narrow width for tools
                        enableTabStrip: false,
                        id: "tools-area",
                        children: [{ type: "tab", id: "tools", component: "ToolbarNode" }]
                    },
                    {
                        type: "tabset",
                        weight: 80, // AAA: Massive center width for Canvas
                        id: "canvas-area",
                        children: [{ type: "tab", id: "canvas", name: "Camera View", component: "CanvasNode", selected: true }]
                    },
                    {
                        type: "tabset",
                        weight: 3, // AAA: Very narrow width for Art Layers
                        enableTabStrip: false,
                        id: "art-layers-area",
                        children: [{ type: "tab", id: "art-layers", component: "ArtLayerToolbarNode" }]
                    },
                    {
                        type: "tabset",
                        weight: 14, // AAA: Fixed comfortable width for Properties
                        id: "properties-area",
                        children: [
                            { type: "tab", id: "properties", name: "Tool Properties", component: "PropertiesNode" },
                            { type: "tab", id: "layer-properties", name: "Layer Properties", component: "LayerPropertiesNode" }
                        ]
                    }
                ]
            },
            {
                type: "tabset",
                weight: 20, // Bottom section takes 20% of vertical height
                id: "timeline-area",
                children: [{ type: "tab", id: "timeline", name: "Timeline", component: "TimelineNode" }]
            }
        ]
    }
};