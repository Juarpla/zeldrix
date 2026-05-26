"use client";

import React from "react";
import { Skeleton } from "@/components/ui/skeleton";
import { Separator } from "@/components/ui/separator";

export function TemplateSkeleton() {
  return (
    <div className="space-y-4 p-4">
      {/* Header skeleton */}
      <div className="flex items-center gap-3 mb-6">
        <Skeleton className="h-6 w-48" />
        <Skeleton className="h-5 w-20 rounded-full" />
      </div>

      <Separator className="mb-6" />

      {/* Document structure skeleton */}
      <div className="space-y-3">
        {/* Title line */}
        <Skeleton className="h-4 w-3/4" />
        <Skeleton className="h-4 w-1/2" />

        {/* Paragraph lines */}
        <div className="pt-2 space-y-2">
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-5/6" />
        </div>

        {/* Variable placeholder skeleton */}
        <div className="flex items-center gap-2 pt-1">
          <Skeleton className="h-5 w-24 rounded" />
          <Skeleton className="h-4 w-16" />
          <Skeleton className="h-5 w-28 rounded" />
        </div>

        {/* More text lines */}
        <div className="pt-2 space-y-2">
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-4/5" />
          <Skeleton className="h-3 w-3/4" />
        </div>

        {/* Another variable */}
        <div className="flex items-center gap-2 pt-1">
          <Skeleton className="h-5 w-20 rounded" />
          <Skeleton className="h-4 w-24" />
        </div>

        {/* Final paragraphs */}
        <div className="pt-2 space-y-2">
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-2/3" />
        </div>
      </div>

      {/* Footer skeleton */}
      <div className="pt-4 space-y-2">
        <Skeleton className="h-3 w-1/3" />
        <Skeleton className="h-3 w-1/4" />
      </div>
    </div>
  );
}