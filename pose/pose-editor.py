import tkinter as tk
from tkinter import filedialog, ttk
import json
import math
import numpy as np

class PoseEditor:
    def __init__(self, root):
        self.root = root
        self.root.title("Stick Figure Pose Editor")
        self.root.geometry("1000x700")
        
        # Default pose values
        self.pose = {
            "facing": True,
            "head": 0.0,
            "body": 0.0,
            "right_upper_arm": 10.0,
            "right_lower_arm": 90.0,
            "right_upper_leg": 10.0,
            "right_lower_leg": -40.0,
            "left_upper_arm": 30.0,
            "left_lower_arm": 90.0,
            "left_upper_leg": 40.0,
            "left_lower_leg": -50.0
        }
        
        # Canvas for drawing the stick figure
        self.canvas_frame = tk.Frame(root)
        self.canvas_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        
        self.canvas = tk.Canvas(self.canvas_frame, bg="white", width=600, height=600)
        self.canvas.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        
        # Control panel
        self.control_frame = tk.Frame(root, width=300)
        self.control_frame.pack(side=tk.RIGHT, fill=tk.Y, padx=10, pady=10)
        
        # Facing direction
        self.facing_var = tk.BooleanVar(value=self.pose["facing"])
        self.facing_frame = tk.Frame(self.control_frame)
        self.facing_frame.pack(fill=tk.X, pady=5)
        
        tk.Label(self.facing_frame, text="Facing:").pack(side=tk.LEFT)
        tk.Radiobutton(self.facing_frame, text="Right", variable=self.facing_var, 
                       value=True, command=self.update_figure).pack(side=tk.LEFT)
        tk.Radiobutton(self.facing_frame, text="Left", variable=self.facing_var, 
                       value=False, command=self.update_figure).pack(side=tk.LEFT)
        
        # Sliders for all pose components
        self.sliders = {}
        self.create_sliders()
        
        # Button frame
        self.button_frame = tk.Frame(self.control_frame)
        self.button_frame.pack(fill=tk.X, pady=10)
        
        # Save and load buttons
        tk.Button(self.button_frame, text="Save Pose", command=self.save_pose).pack(side=tk.LEFT, padx=5)
        tk.Button(self.button_frame, text="Load Pose", command=self.load_pose).pack(side=tk.LEFT, padx=5)
        tk.Button(self.button_frame, text="Reset", command=self.reset_pose).pack(side=tk.LEFT, padx=5)
        
        # Initialize the drawing
        self.update_figure()
    
    def create_sliders(self):
        # Create sliders for each body part
        parts = [key for key in self.pose.keys() if key != "facing"]
        
        for part in parts:
            frame = tk.Frame(self.control_frame)
            frame.pack(fill=tk.X, pady=2)
            
            tk.Label(frame, text=f"{part.replace('_', ' ').title()}:").pack(side=tk.LEFT)
            
            var = tk.DoubleVar(value=self.pose[part])
            slider = ttk.Scale(frame, from_=-180, to=180, variable=var, 
                              orient=tk.HORIZONTAL, length=200, 
                              command=lambda v, p=part: self.update_angle(p, v))
            slider.pack(side=tk.LEFT, padx=5, fill=tk.X, expand=True)
            
            value_label = tk.Label(frame, text=f"{self.pose[part]:.1f}°", width=6)
            value_label.pack(side=tk.LEFT)
            
            self.sliders[part] = {
                "var": var,
                "slider": slider,
                "label": value_label
            }
    
    def update_angle(self, part, value):
        value = float(value)
        self.pose[part] = value
        self.sliders[part]["label"].config(text=f"{value:.1f}°")
        self.update_figure()
    
    def update_figure(self):
        # Update the facing direction
        self.pose["facing"] = self.facing_var.get()
        
        # Clear the canvas
        self.canvas.delete("all")
        
        # Draw the background grid
        self.draw_grid()
        
        # Set up the drawing coordinates
        canvas_width = self.canvas.winfo_width()
        canvas_height = self.canvas.winfo_height()
        center_x = canvas_width // 2
        center_y = canvas_height // 2
        
        # Scale factor for better visibility
        scale = 3
        
        # Starting position (neck)
        neck_x, neck_y = center_x, center_y - 50
        
        # Draw body (body angle affects the entire figure)
        body_angle = -self.pose["body"]
        body_length = 50 * scale
        body_end_x = neck_x + body_length * math.sin(math.radians(body_angle))
        body_end_y = neck_y + body_length * math.cos(math.radians(body_angle))
        
        self.canvas.create_line(neck_x, neck_y, body_end_x, body_end_y, width=3, fill="black", tags="body")
        
        # Draw head
        head_radius = 15 * scale
        head_angle = -self.pose["head"] + body_angle
        head_x = neck_x - head_radius * math.sin(math.radians(head_angle))
        head_y = neck_y - head_radius * math.cos(math.radians(head_angle))
        
        self.canvas.create_oval(head_x - head_radius, head_y - head_radius, 
                               head_x + head_radius, head_y + head_radius, 
                               outline="black", width=2, tags="head")
        
        # Draw limbs
        flip = -1 if self.pose["facing"] else 1
        
        # Arms
        self.draw_limb("right_upper_arm", "right_lower_arm", neck_x, neck_y, body_angle, scale, flip)
        self.draw_limb("left_upper_arm", "left_lower_arm", neck_x, neck_y, body_angle, scale, flip)
        
        # Legs
        self.draw_limb("right_upper_leg", "right_lower_leg", body_end_x, body_end_y, body_angle, scale, flip)
        self.draw_limb("left_upper_leg", "left_lower_leg", body_end_x, body_end_y, body_angle, scale, flip)
    
    def draw_limb(self, upper_part, lower_part, start_x, start_y, parent_angle, scale, flip):
        # Upper part
        upper_angle = -self.pose[upper_part] * flip + parent_angle
        upper_length = 40 * scale
        joint_x = start_x + upper_length * math.sin(math.radians(upper_angle))
        joint_y = start_y + upper_length * math.cos(math.radians(upper_angle))
        
        self.canvas.create_line(start_x, start_y, joint_x, joint_y, width=3, fill="blue", tags=upper_part)
        
        # Lower part
        lower_angle = -self.pose[lower_part] * flip + upper_angle
        lower_length = 40 * scale
        end_x = joint_x + lower_length * math.sin(math.radians(lower_angle))
        end_y = joint_y + lower_length * math.cos(math.radians(lower_angle))
        
        self.canvas.create_line(joint_x, joint_y, end_x, end_y, width=3, fill="red", tags=lower_part)
        
        # Draw joints
        self.canvas.create_oval(joint_x-4, joint_y-4, joint_x+4, joint_y+4, fill="black")
    
    def draw_grid(self):
        width = self.canvas.winfo_width()
        height = self.canvas.winfo_height()
        
        # Draw grid lines
        for i in range(0, width, 50):
            self.canvas.create_line(i, 0, i, height, fill="#DDDDDD", tags="grid")
        
        for i in range(0, height, 50):
            self.canvas.create_line(0, i, width, i, fill="#DDDDDD", tags="grid")
    
    def save_pose(self):
        file_path = filedialog.asksaveasfilename(
            defaultextension=".json",
            filetypes=[("JSON files", "*.json"), ("All files", "*.*")],
            title="Save Pose"
        )
        
        if file_path:
            with open(file_path, 'w') as f:
                json.dump(self.pose, f, indent=4)
    
    def load_pose(self):
        file_path = filedialog.askopenfilename(
            filetypes=[("JSON files", "*.json"), ("All files", "*.*")],
            title="Load Pose"
        )
        
        if file_path:
            with open(file_path, 'r') as f:
                new_pose = json.load(f)
                
                # Update the pose
                self.pose = new_pose
                
                # Update UI elements
                self.facing_var.set(self.pose["facing"])
                
                # Update sliders
                for part, value in self.pose.items():
                    if part != "facing" and part in self.sliders:
                        self.sliders[part]["var"].set(value)
                        self.sliders[part]["label"].config(text=f"{value:.1f}°")
                
                # Update the figure
                self.update_figure()
    
    def reset_pose(self):
        # Reset to default idle pose
        default_pose = {
            "facing": True,
            "head": 0.0,
            "body": 0.0,
            "right_upper_arm": 10.0,
            "right_lower_arm": 90.0,
            "right_upper_leg": 10.0,
            "right_lower_leg": -40.0,
            "left_upper_arm": 30.0,
            "left_lower_arm": 90.0,
            "left_upper_leg": 40.0,
            "left_lower_leg": -50.0
        }
        
        # Update the pose
        self.pose = default_pose
        
        # Update UI elements
        self.facing_var.set(self.pose["facing"])
        
        # Update sliders
        for part, value in self.pose.items():
            if part != "facing" and part in self.sliders:
                self.sliders[part]["var"].set(value)
                self.sliders[part]["label"].config(text=f"{value:.1f}°")
        
        # Update the figure
        self.update_figure()

if __name__ == "__main__":
    root = tk.Tk()
    app = PoseEditor(root)
    
    # Make the canvas respond to window resizing
    def on_resize(event):
        app.update_figure()
    
    root.bind("<Configure>", on_resize)
    root.mainloop()