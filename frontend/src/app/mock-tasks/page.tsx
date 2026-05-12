"use client";

import { TaskDetailView } from "@/components/task/TaskDetail";
import makeMockTasks from "@/lib/mocks/mockTasks";
import { useState } from "react";

export default function MockTasksPage() {
  const items = makeMockTasks();
  const [refreshKey, setRefreshKey] = useState(0);

  return (
    <div className="mx-auto max-w-5xl px-4 py-8 space-y-8">
      <h2 className="text-2xl font-bold text-white">Mock Tasks</h2>
      {items.map(({ task, pubkey }) => (
        <div key={pubkey.toBase58()} className="rounded-lg border border-white/[0.08] bg-surface-card">
          <TaskDetailView
            task={task}
            taskPubkey={pubkey}
            onSuccess={() => setRefreshKey((k) => k + 1)}
          />
        </div>
      ))}
    </div>
  );
}
