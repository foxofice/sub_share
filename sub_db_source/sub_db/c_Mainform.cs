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
	public partial class c_Mainform : Form
	{
		public c_Mainform()
		{
			InitializeComponent();
		}

		internal static c_Mainform	m_s_mainform				= null;
		internal c_UpdateDatabase	m_UpdateDatabase			= new c_UpdateDatabase();
		internal c_Search			m_Search					= new c_Search();
		internal c_Setting			m_Setting					= new c_Setting();
		internal c_About			m_About						= new c_About();

		// DataGridView 的事件是否有效
		internal bool				m_DataGridView_event_enable	= true;

		#region Winform 事件
		/*==============================================================
		 * 窗口加载时
		 *==============================================================*/
		private void c_Mainform_Load(object sender, EventArgs e)
		{
			m_s_mainform	= this;

			this.Icon		= c_Image_.img2icon(Resource1.Logo);
			this.Text		= $"{c_Common_.m_k_PROGRAM_NAME} {c_Common_.m_k_VERSION}";

			// 读取配置文件
			c_Config_.read_config();

			// 加载多语言模板
			c_Languages_.read_languages_list();
			c_Languages_.change_language_to_default();

			// 加载数据库文件
			c_Data_.read_data_from_file();
		}

		/*==============================================================
		 * 窗口关闭时
		 *==============================================================*/
		private void c_Mainform_FormClosing(object sender, FormClosingEventArgs e)
		{
			if(MessageBox.Show(	c_Languages_.txt(1),	// 是否要退出程序？
								$"{c_Common_.m_k_PROGRAM_NAME} {c_Common_.m_k_VERSION}",
								MessageBoxButtons.YesNo,
								MessageBoxIcon.Question,
								MessageBoxDefaultButton.Button2 ) == DialogResult.No)
			{
				e.Cancel = true;
				return;
			}
		}

		/*==============================================================
		 * 调整窗口结束
		 *==============================================================*/
		private void c_Mainform_ResizeEnd(object sender, EventArgs e)
		{
			c_Config_.m_s_window_width	= this.Width;
			c_Config_.m_s_window_height	= this.Height;
			c_Config_.m_s_window_x		= this.DesktopLocation.X;
			c_Config_.m_s_window_y		= this.DesktopLocation.Y;

			c_Config_.write_config();
		}

		/*==============================================================
		 * 激活窗口时
		 *==============================================================*/
		private void c_Mainform_Activated(object sender, EventArgs e)
		{
			c_Common_.RegHotKey(this.Handle,
								c_Common_.m_k_HOTKEY_OPEN_DIR_ID,
								c_Common_.KeyModifiers.Alt,
								Keys.Enter);

			c_Common_.RegHotKey(this.Handle,
								c_Common_.m_k_HOTKEY_OPEN_URL_ID,
								c_Common_.KeyModifiers.None,
								Keys.F1);
		}

		/*==============================================================
		 * 取消激活窗口时
		 *==============================================================*/
		private void c_Mainform_Deactivate(object sender, EventArgs e)
		{
			c_Common_.UnregisterHotKey(this.Handle, c_Common_.m_k_HOTKEY_OPEN_DIR_ID);
			c_Common_.UnregisterHotKey(this.Handle, c_Common_.m_k_HOTKEY_OPEN_URL_ID);
		}

		/*==============================================================
		 * Win32 消息处理
		 *==============================================================*/
		protected override void WndProc(ref Message msg)
		{
			const int WM_HOTKEY = 0x312;

			base.WndProc(ref msg);
			switch(msg.Msg)
			{
			case WM_HOTKEY:
				switch(msg.WParam.ToInt32())
				{
				case c_Common_.m_k_HOTKEY_OPEN_DIR_ID:
					ToolStripButton_Folder_Click(null, null);
					break;

				case c_Common_.m_k_HOTKEY_OPEN_URL_ID:
					ToolStripButton_URL_Click(null, null);
					break;
				}
				break;

			default:
				break;
			}	// switch
		}

		/*==============================================================
		 * 查找（回车）
		 *==============================================================*/
		private void TextBox_Filter_KeyPress(object sender, KeyPressEventArgs e)
		{
			if(e.KeyChar == '\r')
				do_search();
		}

		/*==============================================================
		 * 查找（按钮）
		 *==============================================================*/
		private void PictureBox_Search_Click(object sender, EventArgs e)
		{
			do_search();
		}

		/*==============================================================
		 * 查询语法帮助
		 *==============================================================*/
		private void LinkLabel_FilterHelp_LinkClicked(object sender, LinkLabelLinkClickedEventArgs e)
		{
			try
			{
				System.Diagnostics.Process.Start("https://www.csharp-examples.net/dataview-rowfilter/");
			}
			catch(Exception)
			{
			}
		}

		/*==============================================================
		 * 调整列宽
		 *==============================================================*/
		private void DataGridView_Main_ColumnWidthChanged(object sender, DataGridViewColumnEventArgs e)
		{
			if(!m_DataGridView_event_enable)
				return;

			for(int i=0; i<(int)c_Data_.e_ColumnName.MAX; ++i)
				c_Config_.m_s_column_width[i] = dataGridView_Main.Columns[((c_Data_.e_ColumnName)i).ToString()].Width;

			c_Config_.write_config();
		}

		/*==============================================================
		 * 调整列顺序
		 *==============================================================*/
		private void DataGridView_Main_ColumnDisplayIndexChanged(object sender, DataGridViewColumnEventArgs e)
		{
			if(!m_DataGridView_event_enable)
				return;

			for(int i=0; i<(int)c_Data_.e_ColumnName.MAX; ++i)
				c_Config_.m_s_column_idx[i] = dataGridView_Main.Columns[((c_Data_.e_ColumnName)i).ToString()].DisplayIndex;

			c_Config_.write_config();
		}
		#endregion

		#region 顶部菜单
		/*==============================================================
		 * 更新数据库
		 *==============================================================*/
		private void ToolStripButton_UpdateDB_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_UpdateDatabase);
		}

		/*==============================================================
		 * 高级查找
		 *==============================================================*/
		private void ToolStripButton_Search_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_Search);
		}

		/*==============================================================
		 * 设置
		 *==============================================================*/
		private void ToolStripButton_Settings_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_Setting);
		}

		/*==============================================================
		 * 关于
		 *==============================================================*/
		private void ToolStripButton_About_Click(object sender, EventArgs e)
		{
			c_Forms_.active_form(m_About);
		}

		/*==============================================================
		 * 打开本地文件夹
		 *==============================================================*/
		private void ToolStripButton_Folder_Click(object sender, EventArgs e)
		{
			if(dataGridView_Main.SelectedCells.Count == 0)
				return;

			DataTable dt = (DataTable)dataGridView_Main.DataSource;
			if(dt == null)
				return;

			DataRow dr = ((DataRowView)dataGridView_Main.SelectedCells[0].OwningRow.DataBoundItem).Row;

			string dir = string.Format("{0:s}/{1:s}/{2:s}/{3:s}",
										c_Config_.m_s_subs_path.Replace("\\", "/"),
										(string)dr[c_Data_.e_ColumnName.path.ToString()],
										(string)dr[c_Data_.e_ColumnName.source.ToString()],
										(string)dr[c_Data_.e_ColumnName.sub_name.ToString()]);

			if(!Directory.Exists(dir))
			{
				MessageBox.Show(string.Format(c_Languages_.txt(25), dir));	// 找不到文件夹 {0:s}
				return;
			}

			c_Path_.open_dir(dir);
		}

		/*==============================================================
		 * 打开远程链接
		 *==============================================================*/
		private void ToolStripButton_URL_Click(object sender, EventArgs e)
		{
			if(dataGridView_Main.SelectedCells.Count == 0)
				return;

			DataTable dt = (DataTable)dataGridView_Main.DataSource;
			if(dt == null)
				return;

			DataRow dr = ((DataRowView)dataGridView_Main.SelectedCells[0].OwningRow.DataBoundItem).Row;

			string dir = string.Format("{0:s}/{1:s}/{2:s}",
										(string)dr[c_Data_.e_ColumnName.path.ToString()],
										(string)dr[c_Data_.e_ColumnName.source.ToString()],
										(string)dr[c_Data_.e_ColumnName.sub_name.ToString()]);

			string url = $"https://github.com/foxofice/sub_share/tree/master/subs_list/{dir}";

			try
			{
				System.Diagnostics.Process.Start(url);
			}
			catch(Exception)
			{
			}
		}
		#endregion

		/*==============================================================
		 * 更新 dataGridView_Main 的各列样式
		 *==============================================================*/
		internal void	update_columns_style()
		{
			m_DataGridView_event_enable = false;

			for(int i=0; i<c_Config_.m_s_column_width.Length; ++i)
			{
				if(c_Config_.m_s_column_width[i] > 0)
					dataGridView_Main.Columns[((c_Data_.e_ColumnName)i).ToString()].Width = c_Config_.m_s_column_width[i];
			}

			SortedDictionary<int, string> sort_list = new SortedDictionary<int, string>();

			for(int i=0; i<c_Config_.m_s_column_idx.Length; ++i)
			{
				if(c_Config_.m_s_column_idx[i] >= 0)
					sort_list.Add(c_Config_.m_s_column_idx[i], ((c_Data_.e_ColumnName)i).ToString());
			}

			foreach(var kvp in sort_list)
				dataGridView_Main.Columns[kvp.Value].DisplayIndex = kvp.Key;

			m_DataGridView_event_enable = true;
		}

		/*==============================================================
		 * 根据条件查找数据
		 *==============================================================*/
		internal void	do_search()
		{
			c_Data_.m_s_lock.EnterReadLock();

			if(textBox_Filter.TextLength == 0)
			{
				m_DataGridView_event_enable = false;
				dataGridView_Main.DataSource = c_Data_.m_s_dt;
				update_columns_style();
				m_DataGridView_event_enable = true;
			}
			else
			{
				string filter_string;

				if(radioButton_SearchByName.Checked)
				{
					StringBuilder sb = new StringBuilder();

					string fix_Name = c_SQL.fix_string(textBox_Filter.Text);

					sb.Append($"[name_chs] like '%{fix_Name}%' or ");
					sb.Append($"[name_cht] like '%{fix_Name}%' or ");
					sb.Append($"[name_jp] like '%{fix_Name}%' or ");
					sb.Append($"[name_en] like '%{fix_Name}%' or ");
					sb.Append($"[name_rome] like '%{fix_Name}%'");

					filter_string = sb.ToString();
				}
				else
					filter_string = textBox_Filter.Text;

				try
				{
					DataRow[] dr_list = c_Data_.m_s_dt.Select(filter_string);

					c_Data_.m_s_dt_search.Rows.Clear();
					foreach(DataRow dr in dr_list)
					{
						DataRow dr_tmp = c_Data_.m_s_dt_search.NewRow();
						for(int i=0; i<c_Data_.m_s_dt_search.Columns.Count; ++i)
							dr_tmp[i] = dr[i];

						c_Data_.m_s_dt_search.Rows.Add(dr_tmp);
					}

					m_DataGridView_event_enable = false;
					dataGridView_Main.DataSource = c_Data_.m_s_dt_search;
					update_columns_style();
					m_DataGridView_event_enable = true;
				}
				catch(Exception ex)
				{
					MessageBox.Show(ex.Message);
				}
			}

			c_Data_.m_s_lock.ExitReadLock();

			update_status();
		}

		/*==============================================================
		 * 更新状态栏
		 *==============================================================*/
		internal void	update_status()
		{
			toolStripStatusLabel_ItemsCount.Text = string.Format(	c_Languages_.txt(23),	// {0:d} 条记录
																	dataGridView_Main.Rows.Count );

			DataTable dt = (DataTable)dataGridView_Main.DataSource;
			if(dt != null)
			{
				HashSet<string> movies = new HashSet<string>();

				foreach(DataRow dr in dt.Rows)
					movies.Add((string)dr[c_Data_.e_ColumnName.path.ToString()]);

				toolStripStatusLabel_MovieCount.Text = string.Format(	c_Languages_.txt(24),	// {0:d} 部影片
																		movies.Count );
			}
		}
	};
}	// namespace sub_db
