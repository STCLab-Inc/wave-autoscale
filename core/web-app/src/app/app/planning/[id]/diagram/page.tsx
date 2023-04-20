import PlanningDetailDiagram from './diagram';

export default function PlanningDetailDiagramPage() {
  return (
    <div className="h-full w-full">
      <PlanningDetailDiagram />
    </div>
    // <div className="flex h-full w-full">
    //   <aside className="flex w-80 flex-col border-r border-base-300 bg-base-200">
    //     {/* Header of aside */}
    //     <div className="prose flex h-14 w-full items-center justify-start border-b border-base-300 px-3">
    //       <h4 className="m-0 flex-1">Plans</h4>
    //       <span className="badge">4</span>
    //     </div>
    //     {/* Plan List */}
    //     <div className="flex-1 overflow-y-auto">{/* Plan Item */}</div>
    //   </aside>
    //   <main className="flex flex-1 flex-col">
    //     {/* Plan Detail Header */}
    //     <div className="h-22 prose flex w-full min-w-full flex-col border-b border-base-300 bg-slate-50 p-3">
    //       {/* Plan Title */}
    //       <h3 className="m-0 flex-1">Free Plan Tenancy Autoscaling</h3>
    //       <div className="tabs">
    //         <a className="tab-bordered tab no-underline">Tab 1</a>
    //         <a className="tab-bordered tab tab-active no-underline">Tab 2</a>
    //         <a className="tab-bordered tab no-underline">Tab 3</a>
    //       </div>
    //     </div>
    //   </main>
    // </div>
  );
}
