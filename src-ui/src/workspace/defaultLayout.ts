import { IJsonModel } from "flexlayout-react";

export const defaultLayoutConfig: IJsonModel = {
    global: {
        splitterSize: 6,
        tabEnableClose: false,
        tabEnableRename: false,
        tabSetTabStripHeight: 32,
    },
    borders: [
        {
            type: "border",
            location: "left",
            size: 50,
            children: [
                {
                    type: "tab",
                    enableDrag: false,
                    name: "Tools",
                    component: "ToolbarNode"
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
                children: [
                    {
                        type: "tab",
                        name: "Viewport",
                        component: "CanvasNode",
                        enableClose: false,
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