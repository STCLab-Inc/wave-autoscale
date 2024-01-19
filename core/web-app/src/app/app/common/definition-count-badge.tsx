export default function DefinitionCountBadge({ count }: { count: number }) {
  return (
    <div className="ml-auto flex h-5 items-center justify-center rounded-[10px] bg-wa-gray-500 px-2 text-[10px] font-medium text-wa-gray-50">
      {count}
    </div>
  );
}
