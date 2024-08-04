pub mod rect;
pub mod stack;
pub mod container;
pub mod text;
use glium::{
	glutin::surface::WindowSurface, Display, Frame, 
};
use winit::window::Window;
use crate::view::RenderContext;
extern crate proc_macro;

/// Widget trait that all widgets must inherit from
pub trait Widget {
	fn render(
		&mut self,
		display:&Display<WindowSurface>,
		frame:&mut Frame,
		window:&Window,
		context:&RenderContext,
	);

	// The properties below are empty since there are macros
	// to implement them, might remove them if I find a way.
	
	/// Set the position of the [`Widget`]  
	/// Note that positions start from the upper left 
	/// corner
	fn position(&mut self,x:i32,y:i32){
		//Empty
	}
	
	/// Set the size of the widget
	//fn size(&mut self,width:u32,height:u32){}

	/// Get the size of the widget
	fn get_size(&self) -> (u32,u32){(0,0)}

	fn arrange_widgets(&mut self){}

}

pub trait Drawable{
	fn position(&mut self, x:i32,y:i32); 
	fn get_position(&self) -> (i32,i32); 
	fn size(&mut self,width:u32,height:u32); 
	fn get_size(&self) -> (u32,u32);
}

/// Represents the sizing constraints a widget should have ie.
/// `Fit` the children, `Fill` the parent, `Relative` to the
/// parent as a percentage or a `Fixed` size in pixels
#[derive(Debug,Clone,Copy,PartialEq)] 
pub enum SizeContraint{
	Fill,
	Fit,
	Relative(f32,f32),
	Fixed(u32,u32)
}
