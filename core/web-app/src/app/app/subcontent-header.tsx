import { ReactNode } from 'react';

interface SubContentHeaderProps {
  title: string;
  children?: ReactNode;
  right?: ReactNode;
}

export default function SubContentHeader({
  title,
  children,
  right,
}: SubContentHeaderProps) {
  return (
    <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-t border-gray-200">
      <div className="font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000">
        {title}
      </div>
      <div className="flex px-4">{right}</div>
      {children}
    </div>
  );
}
