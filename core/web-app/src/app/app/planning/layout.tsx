import PlanningSidebar from './sidebar';

export default function PlanningLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-full w-full flex-row">
      <PlanningSidebar />
      {children}
    </div>
  );
}
