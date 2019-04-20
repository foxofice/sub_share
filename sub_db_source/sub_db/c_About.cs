using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;
using System.IO;

namespace sub_db
{
	public partial class c_About : Form
	{
		public c_About()
		{
			InitializeComponent();
		}

		#region Winform 事件
		/*==============================================================
		 * 窗口加载时
		 *==============================================================*/
		private void c_About_Load(object sender, EventArgs e)
		{
			this.Icon				= c_Image_.img2icon(Resource1.Logo);
			this.Text				= string.Format("{0:s} {1:s} {2:s}",
													c_Languages_.txt(14),	// 关于
													c_Common_.m_k_PROGRAM_NAME,
													c_Common_.m_k_VERSION);

			pictureBox_Icon.Image	= Resource1.Logo;

			if(File.Exists(c_Path_.m_k_CHANGELOG_FILENAME))
				textBox_ChangeLog.Text = File.ReadAllText(c_Path_.m_k_CHANGELOG_FILENAME, Encoding.UTF8);
		}

		/*==============================================================
		 * 窗口关闭时
		 *==============================================================*/
		private void c_About_FormClosing(object sender, FormClosingEventArgs e)
		{
			e.Cancel = true;
			this.Hide();
		}

		/*==============================================================
		 * 网站链接
		 *==============================================================*/
		private void LinkLabel_Website_LinkClicked(object sender, LinkLabelLinkClickedEventArgs e)
		{
			try
			{
				System.Diagnostics.Process.Start(linkLabel_Website.Text);
			}
			catch (Exception)
			{
			}
		}

		/*==============================================================
		 * 确定
		 *==============================================================*/
		private void Button_OK_Click(object sender, EventArgs e)
		{
			this.Close();
		}
		#endregion
	};
}	// namespace sub_db
