import React, { useState } from 'react';
import { usePreferencesStore } from '../store/PreferencesStore';

export const LayerPropertiesPanel: React.FC = () => {
    const { setLayerOpacity } = usePreferencesStore();
    const [opacity, setOpacity] = useState(100);

    const handleOpacityChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const val = parseInt(e.target.value);
        setOpacity(val);
        // Mocking layer ID for now
        setLayerOpacity(1000, val / 100.0); 
    };

    return (
        <div className="p-4 flex flex-col gap-6 text-[#dbdee1] text-sm font-sans bg-[#1c1d20] h-full">
            <div>
                <label className="block text-[10px] font-bold text-[#888] mb-2 uppercase tracking-wider">Layer Opacity</label>
                <div className="flex items-center gap-3">
                    <input 
                        type="range" min="0" max="100" value={opacity} 
                        onChange={handleOpacityChange}
                        className="flex-1 accent-[#4752c4] cursor-pointer"
                    />
                    <span className="w-10 text-right text-xs bg-[#141517] border border-[#2a2c30] rounded py-1 px-2">{opacity}%</span>
                </div>
            </div>
            <div>
                <label className="block text-[10px] font-bold text-[#888] mb-2 uppercase tracking-wider">Blend Mode</label>
                <select className="w-full bg-[#141517] border border-[#2a2c30] rounded px-3 py-2 text-xs outline-none focus:border-[#4752c4] cursor-pointer">
                    <option>Normal</option>
                    <option>Multiply</option>
                    <option>Screen</option>
                    <option>Add</option>
                </select>
            </div>
        </div>
    );
};