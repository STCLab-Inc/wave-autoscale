'use client';

import React from 'react';
import Image from 'next/image';
import classNames from 'classnames';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

function getTextMenuClassNames(currentPathname: string, targetPath?: string) {
  return classNames(
    !targetPath || currentPathname.indexOf(targetPath) < 0
      ? 'text-gray-600'
      : 'text-gray-1000'
  );
}

function isActivePath(currentPathname: string, targetPath: string) {
  return currentPathname.includes(targetPath);
}

function MenuItem({
  pathname,
  targetPath,
  label,
  responsive,
}: {
  pathname: string;
  targetPath: string;
  label: string;
  responsive?: boolean;
}) {
  const responsiveFlag = responsive ?? true;

  return (
    <li
      className={classNames(
        'flex-column mx-0.5 flex h-full items-center lg:flex-row',
        isActivePath(pathname, targetPath)
          ? 'border-b-4 border-purple-500'
          : 'border-b-4 border-white'
      )}
    >
      <Link
        className={
          classNames(
            getTextMenuClassNames(pathname, targetPath),
            'whitespace-nowrap px-4'
          ) + (!responsiveFlag ? ' py-2' : '')
        }
        href={targetPath}
      >
        {label}
      </Link>
    </li>
  );
}

export default function Menu({
  windowWidth,
  menuFlag,
  setMenuFlag,
  responsive,
}: {
  windowWidth: number;
  menuFlag: boolean;
  setMenuFlag: React.Dispatch<React.SetStateAction<boolean>>;
  responsive: boolean;
}) {
  const pathname = usePathname();

  return windowWidth < 880 ? (
    responsive ? (
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
    ) : (
      <ul className="flex h-full flex-col items-center lg:flex-row">
        <MenuItem
          pathname={pathname}
          targetPath="/app/autoscaling-history"
          label="Autoscaling History"
          responsive={responsive}
        />
        <MenuItem
          pathname={pathname}
          targetPath="/app/scaling-plans"
          label="Scaling Plans"
          responsive={responsive}
        />
        <MenuItem
          pathname={pathname}
          targetPath="/app/metrics"
          label="Metrics"
          responsive={responsive}
        />
        <MenuItem
          pathname={pathname}
          targetPath="/app/scaling-components"
          label="Scaling Components"
          responsive={responsive}
        />
        <MenuItem
          pathname={pathname}
          targetPath="/app/inflow"
          label="Inflow"
          responsive={responsive}
        />
        <li className="flex-column mx-0.5 flex h-full items-center lg:flex-row">
          <Link
            className={classNames(
              getTextMenuClassNames(pathname),
              'whitespace-nowrap px-4 py-2'
            )}
            href="https://github.com/STCLab-Inc/wave-autoscale"
            target="_blank"
          >
            Github
          </Link>
        </li>
      </ul>
    )
  ) : (
    <ul className="flex h-full flex-row items-center pt-1 lg:flex-row">
      <MenuItem
        pathname={pathname}
        targetPath="/app/autoscaling-history"
        label="Autoscaling History"
      />
      <MenuItem
        pathname={pathname}
        targetPath="/app/scaling-plans"
        label="Scaling Plans"
      />
      <MenuItem pathname={pathname} targetPath="/app/metrics" label="Metrics" />
      <MenuItem
        pathname={pathname}
        targetPath="/app/scaling-components"
        label="Scaling Components"
      />
      <MenuItem pathname={pathname} targetPath="/app/inflow" label="Inflow" />
      <li className="flex-column mx-0.5 flex h-full items-center lg:flex-row">
        <Link
          className={classNames(
            getTextMenuClassNames(pathname),
            'whitespace-nowrap px-4'
          )}
          href="https://github.com/STCLab-Inc/wave-autoscale"
          target="_blank"
        >
          Github
        </Link>
      </li>
    </ul>
  );
}
