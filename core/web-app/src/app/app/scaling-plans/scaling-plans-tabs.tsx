import React from 'react';
import classNames from 'classnames';

function getTabClassNames(isActive: boolean) {
  return classNames(
    'flex-column mx-0.5 flex h-full items-center lg:flex-row px-4',
    isActive
      ? 'border-b-4 border-blue-400 text-gray-1000'
      : 'border-b-4 border-white text-gray-600'
  );
}
export default function ScalingPlansTabs<T>({
  tabItems,
  selected,
  onChange,
}: {
  tabItems: { viewMode: T; label: string }[];
  selected: T;
  onChange: (viewMode: T) => void;
}) {
  return (
    <div className="min-h-12 flex h-12 w-full items-center border-b px-4">
      <ul className="flex h-full flex-row items-center pt-1 lg:flex-row">
        {tabItems.map((tabItem, index) => {
          let isActive = tabItem.viewMode === selected;
          return (
            <button
              key={index}
              className={getTabClassNames(isActive)}
              onClick={() => onChange(tabItem.viewMode)}
            >
              <span>{tabItem.label}</span>
            </button>
          );
        })}
      </ul>
    </div>
  );
}
