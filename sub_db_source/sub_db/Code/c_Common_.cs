using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Microsoft.WindowsAPICodePack.Taskbar;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace sub_db
{
	internal class c_Common_
	{
		internal const string	m_k_PROGRAM_NAME		= "Subtitles Database";
		internal const string	m_k_VERSION				= "v0.09";

		internal const int		m_k_MAX_PROGRESS_VALUE	= 10000;

		internal const int		m_k_HOTKEY_OPEN_DIR_ID	= 1;	// 「打开本地文件夹」的热键 ID
		internal const int		m_k_HOTKEY_OPEN_URL_ID	= 2;	// 「打开远程链接」的热键 ID

		[Flags()]
		public enum KeyModifiers { None = 0, Alt = 1, Ctrl = 2, Shift = 4, WindowsKey = 8 }

		[DllImport("user32.dll", SetLastError = true)]
		public static extern bool RegisterHotKey(IntPtr hWnd, int id, KeyModifiers fsModifiers, Keys vk);

		[DllImport("user32.dll", SetLastError = true)]
		public static extern bool UnregisterHotKey(IntPtr hWnd, int id);

		/*==============================================================
		 * 设置任务栏进度
		 *==============================================================*/
		internal static void	SetProgressValue(int currentValue)
		{
			c_Mainform.m_s_mainform.m_UpdateDatabase.progressBar_Update.Value = currentValue;

			if(Environment.OSVersion.Version.Major >= 6)
			{
				if(TaskbarManager.IsPlatformSupported)
					TaskbarManager.Instance.SetProgressValue(currentValue, m_k_MAX_PROGRESS_VALUE);
			}
		}

		/*==============================================================
		 * 注册热键
		 *==============================================================*/
		internal static void	RegHotKey(IntPtr hwnd, int hotKeyId, KeyModifiers keyModifiers, Keys key)
		{
			if(!RegisterHotKey(hwnd, hotKeyId, keyModifiers, key))
			{
				int errorCode = Marshal.GetLastWin32Error();

				string err_txt = (errorCode == 1409) ? "热键被占用！" : $"注册热键失败！({errorCode})";
				MessageBox.Show(err_txt);
			}
		}

		/*==============================================================
		 * 注销热键
		 *==============================================================*/
		internal static void	UnRegHotKey(IntPtr hwnd, int hotKeyId)
		{
			UnregisterHotKey(hwnd, hotKeyId);
		}
	};
}	// namespace sub_db
