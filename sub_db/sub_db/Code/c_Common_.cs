using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Microsoft.WindowsAPICodePack.Taskbar;

namespace sub_db
{
	internal class c_Common_
	{
		internal const string	m_k_PROGRAM_NAME		= "Subtitles Database";
		internal const string	m_k_VERSION				= "v0.05";

		internal const int		m_k_MAX_PROGRESS_VALUE	= 10000;

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
	};
}	// namespace sub_db
