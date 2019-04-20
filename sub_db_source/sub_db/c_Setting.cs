using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace sub_db
{
	public partial class c_Setting : Form
	{
		public c_Setting()
		{
			InitializeComponent();
		}

		#region Winform 事件
		/*==============================================================
		 * 窗口加载时
		 *==============================================================*/
		private void c_Setting_Load(object sender, EventArgs e)
		{
			this.Icon	= c_Image_.img2icon(Resource1.Logo);

			update_UI();
		}

		/*==============================================================
		 * 窗口关闭时
		 *==============================================================*/
		private void c_Setting_FormClosing(object sender, FormClosingEventArgs e)
		{
			e.Cancel = true;
			this.Hide();

			update_UI();
		}

		/*==============================================================
		 * 浏览
		 *==============================================================*/
		private void Button_subs_path_Click(object sender, EventArgs e)
		{
			FolderBrowserDialog dlg = new FolderBrowserDialog();
			dlg.SelectedPath = textBox_subs_path.Text;

			if(dlg.ShowDialog() == DialogResult.OK)
				textBox_subs_path.Text = dlg.SelectedPath;
		}

		/*==============================================================
		 * 确定
		 *==============================================================*/
		private void Button_OK_Click(object sender, EventArgs e)
		{
			c_Config_.m_s_subs_path = textBox_subs_path.Text;

			c_Config_.write_config();
			this.Close();
		}
		#endregion

		/*==============================================================
		 * 根据设置更新 UI
		 *==============================================================*/
		internal void	update_UI()
		{
			textBox_subs_path.Text = c_Config_.m_s_subs_path;
		}
	};
}	// namespace sub_db
