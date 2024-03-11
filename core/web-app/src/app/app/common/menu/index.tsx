'use client';

import { URLPattern } from 'urlpattern-polyfill';
import React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import MenuItem from './menu-item';
import { MenuStats, useMenuStats } from '@/services/stats';
import classNames from 'classnames';

interface MenuItemData {
  imageFilename: string;
  pathname: string;
  targetPath: string;
  label: string;
  statsKey?: keyof MenuStats;
}

interface MenuItemGroupData {
  groupLabel: string;
}

const MENU_ITEMS: (MenuItemData | MenuItemGroupData)[] = [
  { groupLabel: 'Home' },
  {
    pathname: '/app',
    targetPath: '/app',
    label: 'Dashboard',
    imageFilename: 'dashboard.svg',
  },
  {
    pathname: '/app/templates',
    targetPath: '/app/templates',
    label: 'Templates',
    imageFilename: 'templates.svg',
  },
  {
    pathname: '/app/plan-viewer/:id*',
    targetPath: '/app/plan-viewer',
    label: 'Plan Viewer',
    imageFilename: 'plan-logs.svg',
  },
  {
    pathname: '/app/metrics-viewer/:id*',
    targetPath: '/app/metrics-viewer',
    label: 'Metrics Viewer',
    imageFilename: 'collector-logs.svg',
  },
  { groupLabel: 'Definitions' },
  {
    pathname: '/app/scaling-plans/:id*',
    targetPath: '/app/scaling-plans',
    label: 'Scaling Plans',
    imageFilename: 'scaling-plans.svg',
    statsKey: 'scalingPlansCount',
  },
  {
    pathname: '/app/metrics/:id*',
    targetPath: '/app/metrics',
    label: 'Metrics',
    imageFilename: 'metrics.svg',
    statsKey: 'metricsCount',
  },
  {
    pathname: '/app/scaling-components/:id*',
    targetPath: '/app/scaling-components',
    label: 'Scaling Components',
    imageFilename: 'scaling-components.svg',
    statsKey: 'scalingComponentsCount',
  },
  {
    groupLabel: 'Settings',
  },
  {
    pathname: '/app/wave-config',
    targetPath: '/app/wave-config',
    label: 'wave-config.yaml',
    imageFilename: 'settings.svg',
  },
];

export default function Menu() {
  const pathname = usePathname();
  const { data: stats } = useMenuStats();

  return (
    <ul className="flex h-full w-full flex-col items-center bg-wa-gray-50">
      {MENU_ITEMS.map((item, index) => {
        if ('groupLabel' in item) {
          return (
            <li
              key={index}
              className={classNames(
                'flex h-10 w-full items-center justify-start px-6 text-xs text-wa-gray-900',
                {
                  'border-t border-wa-gray-200': index > 0,
                }
              )}
            >
              {item.groupLabel}
            </li>
          );
        }
        const menuItem = item as MenuItemData;
        const selected = new URLPattern({ pathname: menuItem.pathname }).test({
          pathname,
        });
        return (
          <MenuItem
            key={index}
            iconUrl={`/assets/sidebar/${menuItem.imageFilename.toLowerCase()}`}
            label={menuItem.label}
            targetPath={menuItem.targetPath}
            selected={selected}
            badgeCount={menuItem.statsKey && stats?.[menuItem.statsKey]}
          />
        );
      })}
    </ul>
  );
}
