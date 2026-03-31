import React from 'react';
import { usePreferencesStore } from '../store/PreferencesStore'; // AAA FIX: Case sensitivity

export const PropertiesPanel: React.FC = () => {
    const brush = usePreferencesStore(state => state.brush);
    const setBrushThickness = usePreferencesStore(state => state.setBrushThickness);

    const handleThicknessChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        setBrushThickness(parseFloat(e.target.value));
    };

    return (
        <div style={{ padding: '16px', color: '#E0E0E0', fontFamily: 'sans-serif' }}>
            <h3 style={{ marginTop: 0, marginBottom: '24px', fontSize: '10px', fontWeight: 'bold', textTransform: 'uppercase', letterSpacing: '1px', color: '#555' }}>
                Tool Properties
            </h3>
            <div style={{ marginBottom: '20px' }}>
                <label style={{ display: 'block', marginBottom: '8px', fontSize: '13px', color: '#888' }}>
                    Brush Thickness: {brush.thickness.toFixed(1)}px
                </label>
                <input 
                    type="range" min="1" max="100" step="0.5" 
                    value={brush.thickness}
                    onChange={handleThicknessChange}
                    style={{ width: '100%', cursor: 'pointer', accentColor: '#4752c4' }}
                />
            </div>
            <div style={{ fontSize: '12px', color: '#555' }}>
                <p>Smoothing: {brush.smoothing * 100}%</p>
            </div>
        </div>
    );
};