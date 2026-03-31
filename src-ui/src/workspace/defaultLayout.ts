export const defaultLayoutConfig = {
    global: {
        tabSetEnableTabStrip: true,
        tabSetHeaderHeight: 24,
        tabSetTabStripHeight: 24,
        splitterSize: 2,
        enableEdgeDock: false,
    },
    layout: {
        type: "column", // AAA: Root vertical stack (Top Workspace / Bottom Timeline)
        weight: 100,
        children: [
            // TOP SECTION: The Horizontal Workspace
            {
                type: "row", // AAA: Arranges the panels Side-by-Side
                weight: 75,
                children: [
                    // A. Left Sidebar: Drawing Tools (Fixed Width)
                    {
                        type: "tabset",
                        width: 45,
                        enableTabStrip: false, // Hides the invasive tab label
                        enableDrop: false,
                        enableDrag: false,
                        enableDivide: false,
                        id: "tools-area",
                        children: [{ type: "tab", id: "tools", component: "ToolbarNode" }]
                    },
                    // B. The Center: Camera View (Fluid Weight)
                    {
                        type: "tabset",
                        weight: 100, 
                        id: "canvas-area",
                        enableDrop: false,
                        children: [{ type: "tab", id: "canvas", name: "Camera View", component: "CanvasNode" }]
                    },
                    // C. Center-Right: Art Layer Quadrant (Fixed Width)
                    {
                        type: "tabset",
                        width: 40,
                        enableTabStrip: false,
                        enableDrop: false,
                        enableDrag: false,
                        enableDivide: false,
                        id: "art-layers-area",
                        children: [{ type: "tab", id: "art-layers", component: "ArtLayerToolbarNode" }]
                    },
                    // D. Far Right: Properties Column (Fixed Width)
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
            // BOTTOM SECTION: The Timeline (Full Width Floor)
            {
                type: "tabset",
                weight: 25,
                id: "timeline-area",
                children: [{ type: "tab", id: "timeline", name: "Timeline", component: "TimelineNode" }]
            }
        ]
    }
};