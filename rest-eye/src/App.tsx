import PlayCircleIcon from "@mui/icons-material/PlayCircle";
import RemoveRedEyeIcon from "@mui/icons-material/RemoveRedEye";
import SettingsIcon from "@mui/icons-material/Settings";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Typography from "@mui/material/Typography";
import { invoke } from "@tauri-apps/api/core";
	isPermissionGranted,
	requestPermission,
	sendNotification,
} from "@tauri-apps/plugin-notification";
import { useEffect, useRef, useState } from "react";
import "./App.css";

function App() {
	const timerRef = useRef<number | null>(null);
	const [resting, setResting] = useState(false);
	const [working, setWorking] = useState(false);
	const [workMinutes, setWorkMinutes] = useState(0);
	const [restSeconds, setRestSeconds] = useState(5);

	function startWorkCycle() {
		setWorking(true);
		setWorkMinutes(0);
		setResting(false);
		if (timerRef.current) clearTimeout(timerRef.current);
		const workInterval = window.setInterval(() => {
			setWorkMinutes((m) => m + 1);
		}, 60 * 1000);
		timerRef.current = window.setTimeout(
			async () => {
				clearInterval(workInterval);
				setResting(true);
				setRestSeconds(5);
				let permissionGranted = await isPermissionGranted();
				if (!permissionGranted) {
					const permission = await requestPermission();
					permissionGranted = permission === "granted";
				}
				if (permissionGranted) {
					sendNotification({
						title: "Rest Eye",
						body: "Time to rest your eyes for 5 seconds! Click to acknowledge.",
					});
				}
				const restInterval = window.setInterval(() => {
					setRestSeconds((s) => {
						if (s <= 1) {
							clearInterval(restInterval);
							setResting(false);
							startWorkCycle();
							return 5;
						}
						return s - 1;
					});
				}, 1000);
			},
			20 * 60 * 1000,
		);
	}

	function handleStart() {
		if (!working) startWorkCycle();
	}

	useEffect(() => {
		return () => {
			if (timerRef.current) clearTimeout(timerRef.current);
		};
	}, []);

	return (
		<Box
			sx={{
				display: "flex",
				flexDirection: "column",
				alignItems: "center",
				justifyContent: "center",
				minHeight: "100vh",
				background: "linear-gradient(135deg, #232b5d 0%, #3a6edb 100%)",
			}}
		>
			<Box sx={{ mb: 4, textAlign: "center" }}>
				<RemoveRedEyeIcon
					sx={{
						fontSize: 64,
						color: "#fff",
						mb: 2,
						filter: "drop-shadow(0 0 16px #3a6edb)",
					}}
				/>
				<Typography
					variant="h3"
					sx={{
						color: "#fff",
						fontWeight: 700,
						letterSpacing: 2,
					}}
				>
					Eye Rest
				</Typography>
			</Box>
			<Box
				sx={{
					background: "rgba(255,255,255,0.08)",
					borderRadius: 4,
					boxShadow: 3,
					p: 4,
					mb: 4,
					minWidth: 300,
					textAlign: "center",
				}}
			>
				{!working && (
					<>
						<PauseCircleIcon sx={{ fontSize: 48, color: "#fff", mb: 1 }} />
						<Typography variant="h6" sx={{ color: "#fff", opacity: 0.8 }}>
							Timer Paused
						</Typography>
					</>
				)}
				{working && !resting && (
					<>
						<PlayCircleIcon sx={{ fontSize: 48, color: "#3a6edb", mb: 1 }} />
						<Typography variant="h6" sx={{ color: "#fff", opacity: 0.9 }}>
							Working: {workMinutes} min
						</Typography>
					</>
				)}
				{resting && (
					<>
						<PauseCircleIcon sx={{ fontSize: 48, color: "#ffb300", mb: 1 }} />
						<Typography variant="h6" sx={{ color: "#fff", opacity: 0.9 }}>
							Rest: {restSeconds}s
						</Typography>
					</>
				)}
			</Box>
			<Button
				variant="contained"
				startIcon={<PlayCircleIcon />}
				onClick={handleStart}
				disabled={working}
				sx={{
					background: "linear-gradient(90deg, #3a6edb 0%, #5ad1ff 100%)",
					color: "#fff",
					fontWeight: 700,
					fontSize: 20,
					px: 6,
					py: 2,
					borderRadius: 3,
					boxShadow: 2,
					mb: 2,
					"&:hover": {
						background: "linear-gradient(90deg, #5ad1ff 0%, #3a6edb 100%)",
					},
				}}
			>
				Start
			</Button>
			<Button
				variant="outlined"
				startIcon={<SettingsIcon />}
				sx={{
					color: "#fff",
					borderColor: "#fff",
					fontWeight: 600,
					fontSize: 18,
					px: 4,
					py: 1.5,
					borderRadius: 2,
					boxShadow: 1,
					mt: 1,
					"&:hover": {
						borderColor: "#5ad1ff",
						color: "#5ad1ff",
						background: "rgba(255,255,255,0.12)",
					},
				}}
			>
				Settings
			</Button>
		</Box>
	);
}

export default App;
