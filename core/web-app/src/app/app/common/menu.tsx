'use client';

import React from 'react';

import classNames from 'classnames';
import Image from 'next/image';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

type MenuItemType = 'UNFOLD' | 'FOLD';

interface MenuItemProps {
  type?: MenuItemType;
  isActive: boolean;
  targetPath: string;
  label: string;
  onClick?: () => void;
}

function MenuItem({
  type = 'UNFOLD',
  isActive,
  targetPath,
  label,
  onClick,
}: MenuItemProps) {
  return (
    <li
      className={classNames(
        'flex-column mx-0.5 flex h-full items-center lg:flex-row',
        isActive ? 'border-b-4 border-purple-500' : 'border-b-4 border-white'
      )}
    >
      <Link
        className={classNames(
          isActive ? 'text-gray-1000' : 'text-gray-600',
          'whitespace-nowrap px-4',
          type === 'UNFOLD' ? '' : type === 'FOLD' ? ' py-2' : ''
        )}
        href={targetPath}
        onClick={onClick}
      >
        {label}
      </Link>
    </li>
  );
}

type MenuType = 'UNFOLD' | 'FOLD';

interface MenuProps {
  type?: MenuType;
  windowWidth: number;
  menuFlag?: boolean;
  setMenuFlag: React.Dispatch<React.SetStateAction<boolean>>;
}

const menuItems = [
  { targetPath: '/app/autoscaling-history', label: 'Autoscaling History' },
  { targetPath: '/app/scaling-plans', label: 'Scaling Plans' },
  { targetPath: '/app/metrics', label: 'Metrics' },
  { targetPath: '/app/scaling-components', label: 'Scaling Components' },
  { targetPath: '/app/inflow', label: 'Metrics Inflow' },
  {
    targetPath: 'https://github.com/STCLab-Inc/wave-autoscale',
    label: 'Github',
  },
];

export default function Menu({
  type,
  windowWidth,
  menuFlag,
  setMenuFlag,
}: MenuProps) {
  const pathname = usePathname();

  return windowWidth < 880 ? (
    type === 'UNFOLD' ? (
      <ul className="flex h-full w-full flex-row items-center justify-end pt-1 lg:flex-row">
        <figure
          onClick={() => setMenuFlag((menuFlag) => !menuFlag)}
          className="cursor-pointer"
        >
          <Image
            src="/assets/icons/menu.svg"
            alt="menu.svg"
            priority={true}
            width={36}
            height={32}
            style={{
              minWidth: '1.5rem',
              maxWidth: '1.5rem',
            }}
          />
        </figure>
      </ul>
    ) : type === 'FOLD' ? (
      <ul className="flex h-full flex-col items-center lg:flex-row">
        {menuItems.map((item, index) => (
          <MenuItem
            type={type}
            key={index}
            isActive={pathname.includes(item.targetPath)}
            {...item}
          />
        ))}
      </ul>
    ) : null
  ) : (
    <ul className="flex h-full flex-row items-center pt-1 lg:flex-row">
      {menuItems.map((item, index) => (
        <MenuItem
          type={type}
          key={index}
          isActive={pathname.includes(item.targetPath)}
          {...item}
        />
      ))}
    </ul>
  );
}
