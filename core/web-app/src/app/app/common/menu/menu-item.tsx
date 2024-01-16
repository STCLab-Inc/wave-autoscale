import classNames from 'classnames';
import Link from 'next/link';

interface MenuItemProps {
  iconUrl: string;
  label: string;
  targetPath: string;
  badgeCount?: string;
  selected?: boolean;
}
export default function MenuItem({
  iconUrl,
  label,
  targetPath,
  badgeCount,
  selected,
}: MenuItemProps) {
  return (
    <Link href={targetPath} className="w-full">
      <li
        className={classNames(
          'flex h-12 w-full flex-row items-center justify-start px-6',
          {
            'bg-wa-blue-50': selected,
          }
        )}
      >
        {/* Icon */}
        <div className="mr-2 h-6 w-6">
          <img src={iconUrl} alt={label} className="max-h-full max-w-full" />
        </div>
        {/* Label */}
        <span className="ml-2 text-sm text-wa-gray-700">{label}</span>
        {/* Badge Count */}
        {badgeCount && (
          <div className="ml-auto flex items-center justify-center rounded-[10px] bg-wa-blue-300 text-[10px] font-medium text-wa-gray-50">
            {badgeCount}
          </div>
        )}
      </li>
    </Link>
  );
}
