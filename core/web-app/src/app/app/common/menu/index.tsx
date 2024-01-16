'use client';

import React from 'react';
import classNames from 'classnames';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import MenuItem from './menu-item';

interface MenuItemData {
  imageFilename: string;
  targetPath: string;
  label: string;
}

interface MenuItemGroupData {
  groupLabel: string;
}

const MENU_ITEMS: (MenuItemData | MenuItemGroupData)[] = [
  { groupLabel: 'Home' },
  { targetPath: '/app', label: 'Dashboard', imageFilename: 'dashboard.svg' },
  {
    targetPath: '/app/plan-logs',
    label: 'Plan Logs',
    imageFilename: 'plan-logs.svg',
  },
  {
    targetPath: '/app/collector-logs',
    label: 'Collector Logs',
    imageFilename: 'collector-logs.svg',
  },
  { groupLabel: 'Definitions' },
  {
    targetPath: '/app/scaling-plans',
    label: 'Scaling Plans',
    imageFilename: 'scaling-plans.svg',
  },
  {
    targetPath: '/app/metrics',
    label: 'Metrics',
    imageFilename: 'metrics.svg',
  },
  {
    targetPath: '/app/scaling-components',
    label: 'Scaling Components',
    imageFilename: 'scaling-components.svg',
  },
  // { groupLabel: 'About' },
  // {
  //   targetPath: 'https://github.com/STCLab-Inc/wave-autoscale',
  //   label: 'Github',
  //   imageFilename: 'github.svg',
  // },
];

export default function Menu() {
  const pathname = usePathname();

  return (
    <ul className="flex h-full w-full flex-col items-center bg-wa-gray-50">
      {/* Quick Start */}
      <li className="flex h-14 w-full flex-row items-center justify-start px-6 text-xs text-wa-gray-900">
        <Link
          className="btn-primary btn flex !h-10 !min-h-0 !w-52 items-center justify-center"
          href="/app/templates"
        >
          Quick Start
        </Link>
      </li>
      {MENU_ITEMS.map((item, index) => {
        if ('groupLabel' in item) {
          return (
            <li
              key={index}
              className="flex h-8 w-full items-center justify-start border-t border-wa-gray-200 px-6 text-xs text-wa-gray-900"
            >
              {item.groupLabel}
            </li>
          );
        }
        const menuItem = item as MenuItemData;
        return (
          <MenuItem
            key={index}
            iconUrl={`/assets/sidebar/${menuItem.imageFilename.toLowerCase()}`}
            label={menuItem.label}
            targetPath={menuItem.targetPath}
            selected={pathname === menuItem.targetPath}
          />
        );
      })}
    </ul>
  );
}