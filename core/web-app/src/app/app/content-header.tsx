import classNames from 'classnames';
import { ReactNode } from 'react';

export default function ContentHeader({
  title,
  children,
  right,
}: {
  title: string;
  children?: ReactNode;
  right?: ReactNode;
}) {
  return (
    <div
      className={classNames(
        'flex h-14 w-full min-w-full flex-row items-center justify-between border-b border-t border-gray-200 bg-gray-75 '
      )}
    >
      <div className="font-Pretendard whitespace-nowrap pl-8 pr-4 text-lg font-semibold text-gray-1000">
        {title}
      </div>
      <div className="flex pl-4 pr-8">{right}</div>
      {children}
    </div>
  );
}
