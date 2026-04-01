import React from 'react';
import { Eye, Lock, ChevronRight } from 'lucide-react';
import { usePreferencesStore } from '../store/PreferencesStore';

export const TimelinePanel: React.FC = () => {
    const frameCount = 100;
    const frameWidth = 16; 
    const totalWidth = frameCount * frameWidth;
    const { currentFrame, setCurrentFrame } = usePreferencesStore();
    
    const handleFrameClick = (frame: number) => {
        setCurrentFrame(frame);
    };

    return (
        <div className="flex w-full h-full bg-[#1c1d20] text-[#dbdee1] select-none text-[10px] font-sans overflow-hidden">
            {/* Left Column: Layers Panel */}
            <div style={{ width: '250px', minWidth: '250px' }} className="border-r border-black flex flex-col bg-[#222327] flex-shrink-0 z-30">
                <div className="h-8 border-b border-black flex items-center px-3 bg-[#1c1d20]">
                    <span className="font-bold text-[#555] uppercase tracking-tighter" style={{ whiteSpace: 'nowrap' }}>Layers</span>
                </div>
                <div className="flex items-center h-7 px-2 border-b border-black bg-[#383b40]">
                    <ChevronRight size={14} className="text-[#888] mr-1 flex-shrink-0" />
                    <Eye size={14} className="text-white mr-2 flex-shrink-0" />
                    <Lock size={14} className="text-[#888] mr-2 flex-shrink-0" />
                    <div className="w-3 h-3 bg-blue-500 rounded-sm mr-2 flex-shrink-0" />
                    <span className="truncate flex-1 text-[11px] font-medium" style={{ whiteSpace: 'nowrap' }}>Drawing_1</span>
                </div>
            </div>

            {/* Right Column: Scrolling Timeline Area */}
            <div className="flex-1 flex flex-col overflow-x-auto overflow-y-hidden bg-[#141517] relative">
                
                {/* AAA FIX: Explicitly locked width on the Ruler Track to prevent Flexbox crushing */}
                <div 
                    className="h-8 border-b border-black flex bg-[#1c1d20] sticky top-0 z-20"
                    style={{ width: `${totalWidth}px`, minWidth: `${totalWidth}px` }}
                >
                    {Array.from({ length: frameCount }).map((_, i) => {
                        const isMajor = (i + 1) % 5 === 0;
                        const isCurrent = (i + 1) === currentFrame;
                        return (
                            <div 
                                key={i} 
                                onClick={() => handleFrameClick(i + 1)}
                                style={{ 
                                    width: `${frameWidth}px`, 
                                    minWidth: `${frameWidth}px`, 
                                    backgroundColor: isCurrent ? '#4752c4' : 'transparent', 
                                    cursor: 'pointer' 
                                }} 
                                className="flex flex-col justify-end items-center border-r border-[#222] h-full relative flex-shrink-0 hover:bg-[#2a2d33]"
                            >
                                {isMajor && (
                                    <span className="absolute top-1 font-bold" style={{ fontSize: '9px', left: '50%', transform: 'translateX(-50%)', whiteSpace: 'nowrap', display: 'inline-block', color: isCurrent ? '#fff' : '#888' }}>
                                        {i + 1}
                                    </span>
                                )}
                                <div className={`w-px flex-shrink-0 ${isCurrent ? 'bg-white' : 'bg-[#333]'} ${isMajor ? 'h-2' : 'h-1'}`}></div>
                            </div>
                        );
                    })}
                </div>
                
                {/* AAA FIX: Explicitly locked width on the Exposure Cells Track */}
                <div className="relative flex-1" style={{ width: `${totalWidth}px`, minWidth: `${totalWidth}px` }}>
                    <div 
                        className="absolute top-0 bottom-0 w-px bg-red-600 z-10 pointer-events-none transition-all duration-75" 
                        style={{ left: `${(currentFrame - 1) * frameWidth + (frameWidth * 0.5)}px` }} 
                    />
                    
                    <div className="flex h-7 border-b border-black items-center relative w-full">
                        {/* Placeholder for actual ExposureBlocks. We will map these from Rust shortly. */}
                        <div className="absolute h-4 bg-[#4752c4] rounded-sm border border-[#5865f2]" style={{ left: '1px', width: `${frameWidth * 1}px` }} />
                    </div>
                </div>
            </div>
        </div>
    );
};