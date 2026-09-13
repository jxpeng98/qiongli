<script lang="ts">
	import { Tabs as TabsPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: TabsPrimitive.TriggerProps = $props();
</script>

<TabsPrimitive.Trigger
	bind:ref
	data-slot="tabs-trigger"
	class={cn(
		"gap-1 rounded-[var(--radius-control-inner)] border border-transparent! px-1.5 py-1 text-center text-sm leading-tight font-medium group-data-vertical/tabs:px-2 group-data-vertical/tabs:py-1 [&_svg:not([class*='size-'])]:size-4 relative inline-flex min-h-7 min-w-0 flex-1 items-center justify-center whitespace-normal text-foreground/65 transition-[background-color,border-color,color,box-shadow,transform] duration-150 ease-out group-data-[orientation=vertical]/tabs:w-full group-data-[orientation=vertical]/tabs:justify-start hover:bg-background/70 hover:text-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-1 focus-visible:outline-ring active:scale-[0.985] motion-reduce:transition-none motion-reduce:transform-none disabled:pointer-events-none disabled:opacity-50 dark:text-muted-foreground dark:hover:bg-input/40 dark:hover:text-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0",
		"group-data-[variant=line]/tabs-list:bg-transparent group-data-[variant=line]/tabs-list:data-active:bg-transparent dark:group-data-[variant=line]/tabs-list:data-active:border-transparent dark:group-data-[variant=line]/tabs-list:data-active:bg-transparent",
		"data-active:border-primary! data-active:bg-primary data-active:text-primary-foreground data-active:shadow-sm dark:data-active:border-primary! dark:data-active:bg-primary dark:data-active:text-primary-foreground",
		"after:absolute after:bg-foreground after:opacity-0 after:transition-opacity group-data-[orientation=horizontal]/tabs:after:inset-x-0 group-data-[orientation=horizontal]/tabs:after:bottom-[-5px] group-data-[orientation=horizontal]/tabs:after:h-0.5 group-data-[orientation=vertical]/tabs:after:inset-y-0 group-data-[orientation=vertical]/tabs:after:-right-1 group-data-[orientation=vertical]/tabs:after:w-0.5 group-data-[variant=line]/tabs-list:data-active:after:opacity-100",
		className
	)}
	{...restProps}
/>
