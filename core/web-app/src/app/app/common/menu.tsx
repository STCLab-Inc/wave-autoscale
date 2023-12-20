'use client';

import React from 'react';
import classNames from 'classnames';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

interface MenuItem {
  targetPath?: string;
  label?: string;
  spacer?: boolean;
  isButton?: boolean;
}

const MENU_ITEMS: MenuItem[] = [
  { targetPath: '/app', label: 'Dashboard' },
  { targetPath: '/app/autoscaling-history', label: 'Plan Logs' },
  { targetPath: '/app/inflow', label: 'Metrics Logs' },
  { spacer: true },
  { targetPath: '/app/add-plan', label: 'Add Plan', isButton: true },
  { targetPath: '/app/scaling-plans', label: 'Scaling Plans' },
  { targetPath: '/app/metrics', label: 'Metrics' },
  { targetPath: '/app/scaling-components', label: 'Scaling Components' },
  { spacer: true },
  {
    targetPath: 'https://github.com/STCLab-Inc/wave-autoscale',
    label: 'Github',
  },
];

interface MenuButtonProps {
  menuItem: MenuItem;
  isActive: boolean;
  onClick?: () => void;
}

function MenuItem({ menuItem, isActive, onClick }: MenuButtonProps) {
  if (menuItem.spacer) {
    return <MenuItemSpacer />;
  }
  if (menuItem.isButton) {
    return (
      <MenuItemButton
        menuItem={menuItem}
        isActive={isActive}
        onClick={onClick}
      />
    );
  }
  return (
    <MenuItemDefault
      menuItem={menuItem}
      isActive={isActive}
      onClick={onClick}
    />
  );
}

function MenuItemButton({ menuItem, isActive, onClick }: MenuButtonProps) {
  const targetPath = menuItem.targetPath || '';
  const label = menuItem.label || '';

  return (
    <li
      className={classNames(
        'flex-column mr-4 flex h-full items-center lg:flex-row',
      )}
    >
      <Link
        className={classNames(
          'whitespace-nowrap tracking-tight btn btn-sm btn-primary',
        )}
        href={targetPath}
        onClick={onClick}
        target={targetPath.includes('http') ? '_blank' : ''}
      >
        {label}
      </Link>
    </li>
  );
}

function MenuItemSpacer() {
  return <li className="mx-6 h-4 w-[1px] bg-gray-200" />;
}

function MenuItemDefault({ menuItem, isActive, onClick }: MenuButtonProps) {
  const targetPath = menuItem.targetPath || '';
  const label = menuItem.label || '';

  return (
    <li
      className={classNames(
        'flex-column mr-4 flex h-full items-center lg:flex-row',
        isActive ? 'border-b-4 border-purple-500' : 'border-b-4 border-white'
      )}
    >
      <Link
        className={classNames(
          'whitespace-nowrap px-4 tracking-tight',
          isActive ? 'text-gray-1000' : 'text-gray-600'
        )}
        href={targetPath}
        onClick={onClick}
        target={targetPath.includes('http') ? '_blank' : ''}
      >
        {label}
      </Link>
    </li>
  );
}

interface MenuProps {
  onClickMenu: React.Dispatch<React.SetStateAction<boolean>>;
}

export default function Menu() {
  const pathname = usePathname();

  return (
    <ul className="flex h-full w-full flex-row items-center pt-1 lg:flex-row">
      {MENU_ITEMS.map((menuItem, index) => (
        <MenuItem
          key={index}
          menuItem={menuItem}
          isActive={pathname === menuItem.targetPath}
        />
      ))}
    </ul>
  );
  // return windowWidth < 880 ? (
  //   type === 'UNFOLD' ? (
  //     <ul className="flex h-full w-full flex-row items-center justify-end pt-1 lg:flex-row">
  //       <figure
  //         onClick={() => onClickMenu((menuFlag) => !menuFlag)}
  //         className="cursor-pointer"
  //       >
  //         <Image
  //           src="/assets/icons/menu.svg"
  //           alt="menu.svg"
  //           priority={true}
  //           width={36}
  //           height={32}
  //           style={{
  //             minWidth: '1.5rem',
  //             maxWidth: '1.5rem',
  //           }}
  //         />
  //       </figure>
  //     </ul>
  //   ) : type === 'FOLD' ? (
  //     <ul className="flex h-full flex-col items-center lg:flex-row">
  //       {menuItems.map((item, index) => (
  //         <MenuItem
  //           type={type}
  //           key={index}
  //           isActive={pathname === item.targetPath}
  //           {...item}
  //         />
  //       ))}
  //     </ul>
  //   ) : null
  // ) : (

  // );
}
