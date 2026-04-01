import React, { useRef, useState, useEffect } from 'react';
import { Eye, Lock, ChevronRight } from 'lucide-react';
import { usePreferencesStore } from '../store/PreferencesStore';
import { TimelineInteractionController } from '../engine_bridge/TimelineInteractionController';

export const TimelinePanel: React.FC = () => {
    // AAA FIX: Match Toon Boom Harmony's default scene length
    const frameCount = 60; 
    const frameWidth = 16; 
    const rowHeight = 20; 
    const totalWidth = frameCount * frameWidth;
    
    const { currentFrame, setCurrentFrame, timelineLayers, timelineBlocks } = usePreferencesStore();
    const controller = typeof TimelineInteractionController !== 'undefined' ? TimelineInteractionController.getInstance() : null;
    
    const scrollContainerRef = useRef<HTMLDivElement>(null);
    const [scrollLeft, setScrollLeft] = useState(0);
    const [containerWidth, setContainerWidth] = useState(800);
    const [selectedLayerId, setSelectedLayerId] = useState<bigint | null>(null);

    useEffect(() => {
        if (selectedLayerId === null && timelineLayers.length > 0) {
            setSelectedLayerId(timelineLayers[0].id);
        }
    }, [timelineLayers, selectedLayerId]);

    useEffect(() => {
        const handleResize = () => { if (scrollContainerRef.current) setContainerWidth(scrollContainerRef.current.clientWidth); };
        window.addEventListener('resize', handleResize);
        handleResize();
        return () => window.removeEventListener('resize', handleResize);
    }, []);

    const handleScroll = (e: React.UIEvent<HTMLDivElement>) => { setScrollLeft(e.currentTarget.scrollLeft); };

    const handleCellClick = (frame: number, layerId: bigint) => {
        setCurrentFrame(frame);
        setSelectedLayerId(layerId);
    };

    const visibleStartFrame = Math.max(1, Math.floor(scrollLeft / frameWidth) - 10);
    const visibleEndFrame = Math.min(frameCount, visibleStartFrame + Math.ceil(containerWidth / frameWidth) + 20);

    const rulerTicks = [];
    for (let i = visibleStartFrame; i <= visibleEndFrame; i++) {
        const isMajor = i % 5 === 0;
        const isCurrent = i === currentFrame;
        rulerTicks.push(
            <div key={i} style={{ position: 'absolute', left: `${(i - 1) * frameWidth}px`, width: `${frameWidth}px`, height: '100%', boxSizing: 'border-box', borderRight: '1px solid #2a2a2a' }} 
                className="flex flex-col justify-end items-center">
                
                {isCurrent && (
                    <div style={{ position: 'absolute', top: 0, bottom: 0, left: 0, right: 0, backgroundColor: '#4752c4', opacity: 0.5, zIndex: 0 }} />
                )}

                {isMajor && (
                    <span style={{ position: 'absolute', top: '2px', fontSize: '9px', fontWeight: 'bold', color: isCurrent ? '#fff' : '#888', zIndex: 10 }}>
                        {i}
                    </span>
                )}
                <div style={{ width: '1px', flexShrink: 0, zIndex: 10, backgroundColor: isCurrent ? '#fff' : '#444', height: isMajor ? '8px' : '4px' }}></div>
            </div>
        );
    }

    const gridBackgroundStyle = {
        backgroundImage: `linear-gradient(to right, #2a2a2a 1px, transparent 1px)`,
        backgroundSize: `${frameWidth}px 100%`,
        width: `${totalWidth}px`,
        minWidth: `${totalWidth}px`,
        height: '100%',
        position: 'relative' as const,
    };

    return (
        <div style={{ display: 'flex', width: '100%', height: '100%', backgroundColor: '#1c1d20', userSelect: 'none', fontFamily: 'sans-serif', overflow: 'hidden' }}>
            
            {/* --- Left Column: Fixed Layer Hierarchy --- */}
            <div style={{ width: '250px', minWidth: '250px', borderRight: '1px solid #111', display: 'flex', flexDirection: 'column', backgroundColor: '#222327', flexShrink: 0, zIndex: 30 }}>
                
                <div style={{ height: '32px', minHeight: '32px', borderBottom: '1px solid #111', display: 'flex', alignItems: 'center', padding: '0 12px', backgroundColor: '#1c1d20', boxSizing: 'border-box' }}>
                    <span style={{ fontWeight: 'bold', color: '#666', textTransform: 'uppercase', letterSpacing: '-0.5px', fontSize: '9px' }}>Layers</span>
                </div>
                
                {timelineLayers.length === 0 && (
                    <div style={{ height: `${rowHeight}px`, minHeight: `${rowHeight}px`, display: 'flex', alignItems: 'center', padding: '0 8px', borderBottom: '1px solid #111', backgroundColor: '#383b40', boxSizing: 'border-box' }}>
                         <span style={{ fontSize: '10px', fontWeight: 'bold', color: '#666' }}>Initializing Engine...</span>
                    </div>
                )}
                
                {timelineLayers.map((element) => {
                    const isSelected = selectedLayerId === element.id;
                    return (
                        <div 
                            key={element.id.toString()} 
                            onClick={() => setSelectedLayerId(element.id)}
                            style={{ 
                                height: `${rowHeight}px`, 
                                minHeight: `${rowHeight}px`, 
                                display: 'flex', 
                                alignItems: 'center', 
                                padding: '0 8px', 
                                borderBottom: '1px solid #111', 
                                cursor: 'pointer', 
                                boxSizing: 'border-box',
                                backgroundColor: isSelected ? '#313654' : '#383b40' 
                            }}
                        >
                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '60px', minWidth: '60px', flexShrink: 0, marginRight: '8px' }}>
                                <ChevronRight size={12} color="#888" />
                                <Eye size={12} color="#bbb" />
                                <Lock size={12} color="#666" />
                                <div style={{ width: '8px', height: '8px', backgroundColor: '#3b82f6', borderRadius: '2px' }} />
                            </div>
                            
                            <div style={{ flex: 1, minWidth: 0, overflow: 'hidden' }}>
                                <span style={{ display: 'block', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', fontSize: '11px', fontWeight: 600, color: '#ddd' }}>
                                    {element.name}
                                </span>
                            </div>
                        </div>
                    );
                })}
            </div>

            {/* --- Right Column: The Data Grid --- */}
            <div ref={scrollContainerRef} onScroll={handleScroll} style={{ flex: 1, overflowX: 'auto', overflowY: 'hidden', backgroundColor: '#1a1b1e', position: 'relative' }}>
                
                <div style={gridBackgroundStyle}>
                    
                    {/* Top Ruler Row */}
                    <div 
                        onPointerDown={controller ? controller.handleRulerPointerDown : undefined}
                        onPointerMove={controller ? controller.handleRulerPointerMove : undefined}
                        onPointerUp={controller ? controller.handleRulerPointerUp : undefined}
                        style={{ height: '32px', minHeight: '32px', borderBottom: '1px solid #111', backgroundColor: '#1c1d20', position: 'sticky', top: 0, zIndex: 40, cursor: 'pointer', overflow: 'hidden', width: '100%', boxSizing: 'border-box' }}
                    >
                        {rulerTicks}
                    </div>
                    
                    {/* Global Red Playhead Line */}
                    <div style={{ position: 'absolute', top: 0, bottom: 0, width: '1px', backgroundColor: '#ff4d4d', zIndex: 30, pointerEvents: 'none', transition: 'all 75ms', left: `${(currentFrame - 1) * frameWidth + (frameWidth * 0.5)}px` }} />
                    
                    {/* Exposure Tracks */}
                    {timelineLayers.map((element) => {
                        const isLayerSelected = selectedLayerId === element.id;
                        
                        return (
                            <div 
                                key={element.id.toString()} 
                                style={{ position: 'relative', width: '100%', height: `${rowHeight}px`, minHeight: `${rowHeight}px`, borderBottom: '1px solid #2a2a2a', boxSizing: 'border-box', backgroundColor: isLayerSelected ? '#23252e' : 'transparent' }}
                                onPointerDown={(e) => {
                                    const rect = e.currentTarget.getBoundingClientRect();
                                    const x = e.clientX - rect.left;
                                    const frame = Math.max(1, Math.floor(x / frameWidth) + 1);
                                    handleCellClick(frame, element.id);
                                }}
                            >
                                {timelineBlocks
                                    .filter(b => b.elementId === element.id)
                                    .map(block => (
                                        <div 
                                            key={`${block.start}-${block.duration}`}
                                            style={{ 
                                                position: 'absolute',
                                                display: 'flex',
                                                alignItems: 'center',
                                                zIndex: 10,
                                                boxSizing: 'border-box',
                                                left: `${(block.start - 1) * frameWidth}px`, 
                                                width: `${block.duration * frameWidth}px`,
                                                height: '16px', 
                                                top: '2px', 
                                                backgroundColor: '#c4c4c4', 
                                                borderTop: '1px solid #ffffff', 
                                                borderLeft: '1px solid #ffffff', 
                                                borderBottom: '1px solid #777777', 
                                                borderRight: '1px solid #777777',
                                                borderRadius: '3px',
                                            }}
                                        >
                                            {controller && (
                                                <div 
                                                    onPointerDown={(e) => {
                                                        e.stopPropagation(); 
                                                        controller.handleBlockEdgePointerDown(e, element.id, block.start);
                                                    }}
                                                    onPointerMove={controller.handleBlockEdgePointerMove}
                                                    onPointerUp={controller.handleBlockEdgePointerUp}
                                                    style={{ position: 'absolute', right: 0, top: 0, bottom: 0, width: '8px', cursor: 'ew-resize', zIndex: 20, borderTopRightRadius: '2px', borderBottomRightRadius: '2px' }}
                                                />
                                            )}
                                        </div>
                                    ))
                                }

                                {isLayerSelected && (
                                    <div 
                                        style={{ 
                                            position: 'absolute',
                                            top: 0,
                                            height: '100%', 
                                            zIndex: 20,
                                            pointerEvents: 'none',
                                            boxSizing: 'border-box',
                                            left: `${(currentFrame - 1) * frameWidth}px`, 
                                            width: `${frameWidth}px`,
                                            backgroundColor: 'rgba(0, 168, 255, 0.3)', 
                                            border: '1.5px solid #00a8ff', 
                                        }}
                                    />
                                )}
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
};