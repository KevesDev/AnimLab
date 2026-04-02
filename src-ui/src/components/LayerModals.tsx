import React, { useState } from 'react';
import { useUIStore } from '../store/uiStore';
import { usePreferencesStore } from '../store/PreferencesStore';

export const LayerModals: React.FC = () => {
    const { isAddLayerModalOpen, setAddLayerModalOpen, isDeleteLayerModalOpen, layerToDelete, closeDeleteLayerModal } = useUIStore();
    const { engineInstance, fetchTimelineState, timelineLayers } = usePreferencesStore();
    
    const [layerName, setLayerName] = useState("Drawing");
    const [initMode, setInitMode] = useState<number>(0); 

    const handleAdd = (close: boolean) => {
        if (engineInstance && timelineLayers.length < 100) {
            engineInstance.add_drawing_layer(layerName, initMode);
            fetchTimelineState();
        }
        if (close) setAddLayerModalOpen(false);
        setLayerName("Drawing"); 
    };

    const handleDelete = () => {
        if (engineInstance && layerToDelete !== null) {
            engineInstance.delete_drawing_layer(layerToDelete);
            fetchTimelineState();
        }
        closeDeleteLayerModal();
    };

    return (
        <>
            {isAddLayerModalOpen && (
                <div style={{ position: 'fixed', inset: 0, zIndex: 1000, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.5)', fontFamily: 'sans-serif' }}>
                    <div style={{ backgroundColor: '#222327', border: '1px solid #111', padding: '20px', borderRadius: '4px', width: '400px', boxShadow: '0 10px 25px rgba(0,0,0,0.5)', color: '#ddd' }}>
                        <h2 style={{ fontSize: '14px', fontWeight: 'bold', marginBottom: '16px', color: '#fff' }}>Add Drawing Layer</h2>
                        
                        <label style={{ display: 'block', marginBottom: '8px', fontSize: '10px', fontWeight: 'bold', color: '#888', letterSpacing: '0.5px' }}>NAME</label>
                        <input type="text" value={layerName} onChange={e => setLayerName(e.target.value)} style={{ width: '100%', backgroundColor: '#111', border: '1px solid #333', padding: '6px', marginBottom: '20px', color: 'white', fontSize: '12px', boxSizing: 'border-box', outline: 'none' }} />
                        
                        <label style={{ display: 'block', marginBottom: '8px', fontSize: '10px', fontWeight: 'bold', color: '#888', letterSpacing: '0.5px' }}>EXPOSURE INITIALIZATION</label>
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', marginBottom: '24px' }}>
                            <label style={{ display: 'flex', alignItems: 'center', fontSize: '12px', cursor: 'pointer' }}><input type="radio" checked={initMode===0} onChange={()=>setInitMode(0)} style={{ marginRight: '8px' }}/> Empty Layer</label>
                            <label style={{ display: 'flex', alignItems: 'center', fontSize: '12px', cursor: 'pointer' }}><input type="radio" checked={initMode===1} onChange={()=>setInitMode(1)} style={{ marginRight: '8px' }}/> Single Frame (At current playhead)</label>
                            <label style={{ display: 'flex', alignItems: 'center', fontSize: '12px', cursor: 'pointer' }}><input type="radio" checked={initMode===2} onChange={()=>setInitMode(2)} style={{ marginRight: '8px' }}/> Stretch to Scene</label>
                        </div>
                        
                        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
                            <button onClick={() => setAddLayerModalOpen(false)} style={{ padding: '6px 12px', backgroundColor: '#333', border: 'none', color: '#fff', borderRadius: '2px', cursor: 'pointer', fontSize: '12px' }}>Cancel</button>
                            <button onClick={() => handleAdd(false)} style={{ padding: '6px 12px', backgroundColor: '#4752c4', border: 'none', color: '#fff', borderRadius: '2px', cursor: 'pointer', fontSize: '12px' }}>Add</button>
                            <button onClick={() => handleAdd(true)} style={{ padding: '6px 12px', backgroundColor: '#4752c4', border: 'none', color: '#fff', borderRadius: '2px', cursor: 'pointer', fontSize: '12px', fontWeight: 'bold' }}>Add and Close</button>
                        </div>
                    </div>
                </div>
            )}

            {isDeleteLayerModalOpen && (
                <div style={{ position: 'fixed', inset: 0, zIndex: 1000, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.5)', fontFamily: 'sans-serif' }}>
                    <div style={{ backgroundColor: '#222327', border: '1px solid #111', padding: '20px', borderRadius: '4px', width: '340px', boxShadow: '0 10px 25px rgba(0,0,0,0.5)', color: '#ddd' }}>
                        <h2 style={{ fontSize: '14px', fontWeight: 'bold', marginBottom: '16px', color: '#ff4d4d' }}>Delete Layer?</h2>
                        <p style={{ fontSize: '12px', marginBottom: '24px', lineHeight: '1.5', color: '#bbb' }}>Are you sure? This will instantly delete the layer, its exposures, and all associated artwork vectors. This action can be undone via the Command History.</p>
                        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
                            <button onClick={closeDeleteLayerModal} style={{ padding: '6px 12px', backgroundColor: '#333', border: 'none', color: '#fff', borderRadius: '2px', cursor: 'pointer', fontSize: '12px' }}>Cancel</button>
                            <button onClick={handleDelete} style={{ padding: '6px 12px', backgroundColor: '#e02424', border: 'none', color: '#fff', borderRadius: '2px', cursor: 'pointer', fontSize: '12px', fontWeight: 'bold' }}>Delete Permanently</button>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
}