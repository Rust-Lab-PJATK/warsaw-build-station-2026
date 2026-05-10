import { PostTaskForm } from "@/components/post/PostTaskForm";
import { ArrowLeftIcon } from "lucide-react";
import Link from "next/link";

export default function PostPage() {
  return (
    <div className="mx-auto max-w-xl px-4 py-10">
      <Link
        href="/"
        className="mb-6 inline-flex items-center gap-2 text-sm text-gray-500 hover:text-white transition-colors"
      >
        <ArrowLeftIcon className="h-4 w-4" /> Back
      </Link>

      <h1 className="text-2xl font-bold text-white mb-1">Post a Task</h1>
      <p className="text-sm text-gray-400 mb-8">
        Describe your task and let AI suggest a fair price. Funds are locked in a secure escrow until you approve the result.
      </p>

      <PostTaskForm />
    </div>
  );
}
