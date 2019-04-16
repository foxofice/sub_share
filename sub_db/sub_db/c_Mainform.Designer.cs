namespace sub_db
{
	partial class c_Mainform
	{
		/// <summary>
		/// 必需的设计器变量。
		/// </summary>
		private System.ComponentModel.IContainer components = null;

		/// <summary>
		/// 清理所有正在使用的资源。
		/// </summary>
		/// <param name="disposing">如果应释放托管资源，为 true；否则为 false。</param>
		protected override void Dispose(bool disposing)
		{
			if(disposing && (components != null))
			{
				components.Dispose();
			}
			base.Dispose(disposing);
		}

		#region Windows 窗体设计器生成的代码

		/// <summary>
		/// 设计器支持所需的方法 - 不要修改
		/// 使用代码编辑器修改此方法的内容。
		/// </summary>
		private void InitializeComponent()
		{
			this.components = new System.ComponentModel.Container();
			this.dataGridView_Main = new System.Windows.Forms.DataGridView();
			this.notifyIcon_Main = new System.Windows.Forms.NotifyIcon(this.components);
			this.contextMenuStrip_notifyIcon = new System.Windows.Forms.ContextMenuStrip(this.components);
			this.ToolStripMenuItem_Open = new System.Windows.Forms.ToolStripMenuItem();
			this.ToolStripMenuItem_UpdateDB = new System.Windows.Forms.ToolStripMenuItem();
			this.ToolStripMenuItem_Languages = new System.Windows.Forms.ToolStripMenuItem();
			this.ToolStripMenuItem_Settings = new System.Windows.Forms.ToolStripMenuItem();
			this.ToolStripMenuItem_About = new System.Windows.Forms.ToolStripMenuItem();
			this.toolStripMenuItem1 = new System.Windows.Forms.ToolStripSeparator();
			this.ToolStripMenuItem_Exit = new System.Windows.Forms.ToolStripMenuItem();
			this.textBox_Filter = new System.Windows.Forms.TextBox();
			this.label_Filter = new System.Windows.Forms.Label();
			this.linkLabel_FilterHelp = new System.Windows.Forms.LinkLabel();
			((System.ComponentModel.ISupportInitialize)(this.dataGridView_Main)).BeginInit();
			this.contextMenuStrip_notifyIcon.SuspendLayout();
			this.SuspendLayout();
			// 
			// dataGridView_Main
			// 
			this.dataGridView_Main.AllowUserToAddRows = false;
			this.dataGridView_Main.AllowUserToDeleteRows = false;
			this.dataGridView_Main.AllowUserToOrderColumns = true;
			this.dataGridView_Main.Anchor = ((System.Windows.Forms.AnchorStyles)((((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Bottom) 
            | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
			this.dataGridView_Main.ColumnHeadersHeightSizeMode = System.Windows.Forms.DataGridViewColumnHeadersHeightSizeMode.AutoSize;
			this.dataGridView_Main.Location = new System.Drawing.Point(0, 39);
			this.dataGridView_Main.Name = "dataGridView_Main";
			this.dataGridView_Main.ReadOnly = true;
			this.dataGridView_Main.RowTemplate.Height = 23;
			this.dataGridView_Main.Size = new System.Drawing.Size(784, 522);
			this.dataGridView_Main.TabIndex = 1;
			// 
			// notifyIcon_Main
			// 
			this.notifyIcon_Main.ContextMenuStrip = this.contextMenuStrip_notifyIcon;
			this.notifyIcon_Main.Text = "sub_db";
			this.notifyIcon_Main.Visible = true;
			this.notifyIcon_Main.DoubleClick += new System.EventHandler(this.NotifyIcon_Main_DoubleClick);
			// 
			// contextMenuStrip_notifyIcon
			// 
			this.contextMenuStrip_notifyIcon.Items.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.ToolStripMenuItem_Open,
            this.ToolStripMenuItem_UpdateDB,
            this.ToolStripMenuItem_Languages,
            this.ToolStripMenuItem_Settings,
            this.ToolStripMenuItem_About,
            this.toolStripMenuItem1,
            this.ToolStripMenuItem_Exit});
			this.contextMenuStrip_notifyIcon.Name = "contextMenuStrip_notifyIcon";
			this.contextMenuStrip_notifyIcon.Size = new System.Drawing.Size(181, 164);
			// 
			// ToolStripMenuItem_Open
			// 
			this.ToolStripMenuItem_Open.Font = new System.Drawing.Font("Microsoft YaHei UI", 9F, System.Drawing.FontStyle.Bold);
			this.ToolStripMenuItem_Open.Name = "ToolStripMenuItem_Open";
			this.ToolStripMenuItem_Open.Size = new System.Drawing.Size(180, 22);
			this.ToolStripMenuItem_Open.Text = "打开";
			this.ToolStripMenuItem_Open.Click += new System.EventHandler(this.ToolStripMenuItem_Open_Click);
			// 
			// ToolStripMenuItem_UpdateDB
			// 
			this.ToolStripMenuItem_UpdateDB.Image = global::sub_db.Resource1.DB_Refresh;
			this.ToolStripMenuItem_UpdateDB.Name = "ToolStripMenuItem_UpdateDB";
			this.ToolStripMenuItem_UpdateDB.Size = new System.Drawing.Size(180, 22);
			this.ToolStripMenuItem_UpdateDB.Text = "更新数据库";
			this.ToolStripMenuItem_UpdateDB.Click += new System.EventHandler(this.ToolStripMenuItem_UpdateDB_Click);
			// 
			// ToolStripMenuItem_Languages
			// 
			this.ToolStripMenuItem_Languages.Image = global::sub_db.Resource1.Languages;
			this.ToolStripMenuItem_Languages.Name = "ToolStripMenuItem_Languages";
			this.ToolStripMenuItem_Languages.Size = new System.Drawing.Size(180, 22);
			this.ToolStripMenuItem_Languages.Text = "语言(Languages)";
			// 
			// ToolStripMenuItem_Settings
			// 
			this.ToolStripMenuItem_Settings.Image = global::sub_db.Resource1.Settings;
			this.ToolStripMenuItem_Settings.Name = "ToolStripMenuItem_Settings";
			this.ToolStripMenuItem_Settings.Size = new System.Drawing.Size(180, 22);
			this.ToolStripMenuItem_Settings.Text = "设置";
			this.ToolStripMenuItem_Settings.Click += new System.EventHandler(this.ToolStripMenuItem_Settings_Click);
			// 
			// ToolStripMenuItem_About
			// 
			this.ToolStripMenuItem_About.Image = global::sub_db.Resource1.About;
			this.ToolStripMenuItem_About.Name = "ToolStripMenuItem_About";
			this.ToolStripMenuItem_About.Size = new System.Drawing.Size(180, 22);
			this.ToolStripMenuItem_About.Text = "关于";
			this.ToolStripMenuItem_About.Click += new System.EventHandler(this.ToolStripMenuItem_About_Click);
			// 
			// toolStripMenuItem1
			// 
			this.toolStripMenuItem1.Name = "toolStripMenuItem1";
			this.toolStripMenuItem1.Size = new System.Drawing.Size(177, 6);
			// 
			// ToolStripMenuItem_Exit
			// 
			this.ToolStripMenuItem_Exit.Image = global::sub_db.Resource1.Exit;
			this.ToolStripMenuItem_Exit.Name = "ToolStripMenuItem_Exit";
			this.ToolStripMenuItem_Exit.Size = new System.Drawing.Size(180, 22);
			this.ToolStripMenuItem_Exit.Text = "退出程序";
			this.ToolStripMenuItem_Exit.Click += new System.EventHandler(this.ToolStripMenuItem_Exit_Click);
			// 
			// textBox_Filter
			// 
			this.textBox_Filter.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
			this.textBox_Filter.Location = new System.Drawing.Point(83, 12);
			this.textBox_Filter.Name = "textBox_Filter";
			this.textBox_Filter.Size = new System.Drawing.Size(666, 21);
			this.textBox_Filter.TabIndex = 2;
			// 
			// label_Filter
			// 
			this.label_Filter.AutoSize = true;
			this.label_Filter.Location = new System.Drawing.Point(12, 15);
			this.label_Filter.Name = "label_Filter";
			this.label_Filter.Size = new System.Drawing.Size(65, 12);
			this.label_Filter.TabIndex = 3;
			this.label_Filter.Text = "查询语句：";
			// 
			// linkLabel_FilterHelp
			// 
			this.linkLabel_FilterHelp.Anchor = ((System.Windows.Forms.AnchorStyles)((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Right)));
			this.linkLabel_FilterHelp.AutoSize = true;
			this.linkLabel_FilterHelp.Location = new System.Drawing.Point(755, 15);
			this.linkLabel_FilterHelp.Name = "linkLabel_FilterHelp";
			this.linkLabel_FilterHelp.Size = new System.Drawing.Size(17, 12);
			this.linkLabel_FilterHelp.TabIndex = 4;
			this.linkLabel_FilterHelp.TabStop = true;
			this.linkLabel_FilterHelp.Text = "？";
			this.linkLabel_FilterHelp.LinkClicked += new System.Windows.Forms.LinkLabelLinkClickedEventHandler(this.LinkLabel_FilterHelp_LinkClicked);
			// 
			// c_Mainform
			// 
			this.AutoScaleDimensions = new System.Drawing.SizeF(6F, 12F);
			this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
			this.ClientSize = new System.Drawing.Size(784, 561);
			this.Controls.Add(this.linkLabel_FilterHelp);
			this.Controls.Add(this.label_Filter);
			this.Controls.Add(this.textBox_Filter);
			this.Controls.Add(this.dataGridView_Main);
			this.Name = "c_Mainform";
			this.StartPosition = System.Windows.Forms.FormStartPosition.CenterScreen;
			this.Text = "Subtitles DataBase";
			this.FormClosing += new System.Windows.Forms.FormClosingEventHandler(this.c_Mainform_FormClosing);
			this.Load += new System.EventHandler(this.c_Mainform_Load);
			((System.ComponentModel.ISupportInitialize)(this.dataGridView_Main)).EndInit();
			this.contextMenuStrip_notifyIcon.ResumeLayout(false);
			this.ResumeLayout(false);
			this.PerformLayout();

		}

		#endregion
		private System.Windows.Forms.DataGridView dataGridView_Main;
		private System.Windows.Forms.ContextMenuStrip contextMenuStrip_notifyIcon;
		private System.Windows.Forms.ToolStripSeparator toolStripMenuItem1;
		private System.Windows.Forms.TextBox textBox_Filter;
		internal System.Windows.Forms.NotifyIcon notifyIcon_Main;
		private System.Windows.Forms.LinkLabel linkLabel_FilterHelp;
		internal System.Windows.Forms.Label label_Filter;
		internal System.Windows.Forms.ToolStripMenuItem ToolStripMenuItem_Open;
		internal System.Windows.Forms.ToolStripMenuItem ToolStripMenuItem_UpdateDB;
		internal System.Windows.Forms.ToolStripMenuItem ToolStripMenuItem_Languages;
		internal System.Windows.Forms.ToolStripMenuItem ToolStripMenuItem_Settings;
		internal System.Windows.Forms.ToolStripMenuItem ToolStripMenuItem_About;
		internal System.Windows.Forms.ToolStripMenuItem ToolStripMenuItem_Exit;
	}
}

