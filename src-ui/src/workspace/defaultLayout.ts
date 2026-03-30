import { IJsonModel } from "flexlayout-react";

export const defaultLayoutConfig: IJsonModel = {
    global: {
        splitterSize: 6,
        tabEnableClose: false,
        tabEnableRename: false,
        tabSetTabStripHeight: 32,
        
        // Explicitly declare the engine's global drop rules to validate hit-testing
        tabEnableDrag: true,
        tabSetEnableDrop: true,
        tabSetEnableDivide: true,
    },
    borders: [
        {
            type: "border",
            location: "left",
            size: 50,
            children: [
                {
                    type: "tab",
                    name: "Tools",
                    component: "ToolbarNode",
                    enableDrag: false // The vertical toolbar stays strictly pinned
                }
            ]
        }
    ],
    layout: {
        type: "row",
        weight: 100,
        children: [
            {
                type: "tabset",
                weight: 80,
                id: "canvas-area",
                children: [
                    {
                        type: "tab",
                        name: "Viewport",
                        component: "CanvasNode",
                        enableClose: false,
                        enableDrag: false // The central canvas remains the immovable anchor
                    }
                ]
            },
            {
                type: "tabset",
                weight: 20,
                children: [
                    {
                        type: "tab",
                        name: "Tool Properties",
                        component: "PropertiesNode",
                        enableClose: false,
                    }
                ]
            }
        ]
    }
};