'use client';

import React, { useEffect } from 'react';
import classNames from 'classnames';
import { useParams, usePathname, useRouter } from 'next/navigation';

function getTabClassNames(isActive: boolean) {
  return classNames(
    'flex-column mx-0.5 flex h-full items-center lg:flex-row',
    isActive
      ? 'border-b-4 border-blue-400 text-gray-1000 pr-4 pl-4'
      : 'border-b-4 border-white text-gray-600 pr-4 pl-4'
  );
}

function TabItem({
  isActive,
  label,
  viewMode,
  setViewMode,
}: {
  isActive: boolean;
  label: string;
  viewMode: string;
  setViewMode: (viewMode: string) => void;
}) {
  const onClickTab = () => {
    setViewMode(viewMode);
  };

  return (
    <button className={getTabClassNames(isActive)} onClick={onClickTab}>
      <span>{label}</span>
    </button>
  );
}

export default function ScalingPlansTabs({
  viewMode,
  setViewMode,
}: {
  viewMode?: string;
  setViewMode: (viewMode: string) => void;
}) {
  const tabItems = [
    { viewMode: 'diagram', label: 'Diagram' },
    { viewMode: 'code', label: 'Code' },
  ];

  return (
    <div className="min-h-12 flex h-12 w-full items-center border-b px-4">
      <ul className="flex h-full flex-row items-center pt-1 lg:flex-row">
        {tabItems.map((tabItem, index) => (
          <TabItem
            key={index}
            isActive={tabItem.viewMode === viewMode}
            label={tabItem.label}
            viewMode={tabItem.viewMode}
            setViewMode={setViewMode}
          />
        ))}
      </ul>
    </div>
  );
}
