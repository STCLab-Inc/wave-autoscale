'use client';
import classNames from 'classnames';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

function getMenuClassNames(currentPathname: string, targetPath?: string) {
  return classNames(
    !targetPath || currentPathname.indexOf(targetPath) < 0
      ? 'text-gray-600 pr-4 pl-4'
      : 'text-gray-1000 pr-4 pl-4'
  );
}

function MenuItem({
  pathname,
  targetPath,
  label,
}: {
  pathname: string;
  targetPath: string;
  label: string;
}) {
  return (
    <li
      className={classNames(
        'flex-column flex h-full items-center lg:flex-row',
        pathname.includes(targetPath)
          ? 'border-b-4 border-purple-500'
          : 'border-b-4 border-white'
      )}
    >
      <Link
        className={classNames(
          getMenuClassNames(pathname, targetPath),
          'whitespace-nowrap'
        )}
        href={targetPath}
      >
        {label}
      </Link>
    </li>
  );
}

export default function Menu() {
  const pathname = usePathname();

  return (
    <ul className="flex h-full flex-row items-center pt-1 lg:flex-row">
      <MenuItem
        pathname={pathname}
        targetPath="/app/autoscaling-history"
        label="Autoscaling History"
      />
      <MenuItem
        pathname={pathname}
        targetPath="/app/planning"
        label="Planning"
      />
      <MenuItem pathname={pathname} targetPath="/app/metrics" label="Metrics" />
      <MenuItem
        pathname={pathname}
        targetPath="/app/scaling-components"
        label="Scaling Components"
      />
      <li className="flex-column flex h-full items-center lg:flex-row">
        <Link
          className={classNames(
            getMenuClassNames(pathname),
            'whitespace-nowrap'
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
