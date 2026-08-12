import React, { useState } from 'react';

interface SectionProps {
  title: string;
  description: string;
  defaultChecked?: boolean;
  onChange?: (checked: boolean) => void;
}

const Options: React.FC<SectionProps> = ({ 
  title, 
  description, 
  defaultChecked = false,
  onChange 
}) => {
  const [isChecked, setIsChecked] = useState(defaultChecked);

  const handleToggle = () => {
    const nextState = !isChecked;
    setIsChecked(nextState);
    if (onChange) {
      onChange(nextState);
    }
  };

  return (
    <div className="flex items-center justify-between py-4 border-b border-gray-800">
      {/* Texto alineado a la izquierda */}
      <div className="flex flex-col pr-4">
        <h3 className="text-sm font-medium text-gray-200">{title}</h3>
        <p className="text-xs text-gray-400 mt-0.5">{description}</p>
      </div>

      {/* Switch adaptado al diseño de tu app */}
      <label className="relative inline-flex items-center cursor-pointer shrink-0">
        <input 
          type="checkbox" 
          checked={isChecked}
          onChange={handleToggle}
          className="sr-only peer" 
        />
        <div className="w-9 h-5 bg-[#2a2d3d] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[#5B8DEF]"></div>
      </label>
    </div> 
  );
};

export default Options;